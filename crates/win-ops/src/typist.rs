use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use shared::{Key, Step};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Text(String),
    Key(Key),
    Combo(Vec<Key>),
    Wait(Duration),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Missing {
    Field(String),
    Setting(String),
}

pub fn resolve(
    steps: &[Step],
    field: impl Fn(&str) -> Option<String>,
    setting: impl Fn(&str) -> Option<String>,
) -> Result<Vec<Action>, Missing> {
    steps
        .iter()
        .map(|step| match step {
            Step::Field(name) => field(name)
                .map(Action::Text)
                .ok_or_else(|| Missing::Field(name.to_string())),
            Step::Setting(name) => setting(name)
                .map(Action::Text)
                .ok_or_else(|| Missing::Setting(name.to_string())),
            Step::Text(text) => Ok(Action::Text(text.to_string())),
            Step::Key(key) => Ok(Action::Key(*key)),
            Step::Combo(keys) => Ok(Action::Combo(keys.clone())),
            Step::Wait(duration) => Ok(Action::Wait(*duration)),
        })
        .collect()
}

pub struct Typist<'a> {
    delay: Duration,
    stop: Option<&'a AtomicBool>,
}

impl<'a> Typist<'a> {
    pub fn new(delay: Duration) -> Self {
        Self { delay, stop: None }
    }

    pub fn with_stop(mut self, stop: &'a AtomicBool) -> Self {
        self.stop = Some(stop);
        self
    }

    pub fn run(&self, actions: &[Action]) -> bool {
        for action in actions {
            if self.stopped() {
                return false;
            }
            let finished = match action {
                Action::Text(text) => self.text(text),
                Action::Key(key) => {
                    self.key(*key);
                    true
                }
                Action::Combo(keys) => {
                    self.combo(keys);
                    true
                }
                Action::Wait(duration) => {
                    thread::sleep(*duration);
                    true
                }
            };
            if !finished {
                return false;
            }
            self.pause();
        }
        true
    }

    pub fn text(&self, text: &str) -> bool {
        for unit in text.encode_utf16() {
            if self.stopped() {
                return false;
            }
            send(&[unicode(unit, false), unicode(unit, true)]);
            self.pause();
        }
        true
    }

    pub fn key(&self, key: Key) {
        match key_vk(key) {
            Some(vk) => send(&[virtual_key(vk, false), virtual_key(vk, true)]),
            None => {
                if let Key::Char(ch) = key {
                    self.text(&ch.to_string());
                }
            }
        }
    }

    pub fn combo(&self, keys: &[Key]) {
        let codes: Vec<u16> = keys.iter().filter_map(|key| key_vk(*key)).collect();
        let presses = codes.iter().map(|vk| virtual_key(*vk, false));
        let releases = codes.iter().rev().map(|vk| virtual_key(*vk, true));
        let inputs: Vec<INPUT> = presses.chain(releases).collect();
        send(&inputs);
    }

    fn stopped(&self) -> bool {
        self.stop.is_some_and(|stop| stop.load(Ordering::Relaxed))
    }

    fn pause(&self) {
        if !self.delay.is_zero() {
            thread::sleep(self.delay);
        }
    }
}

fn key_vk(key: Key) -> Option<u16> {
    Some(match key {
        Key::Tab => 0x09,
        Key::Enter => 0x0D,
        Key::Space => 0x20,
        Key::Escape => 0x1B,
        Key::Backspace => 0x08,
        Key::Delete => 0x2E,
        Key::Up => 0x26,
        Key::Down => 0x28,
        Key::Left => 0x25,
        Key::Right => 0x27,
        Key::Home => 0x24,
        Key::End => 0x23,
        Key::Ctrl => 0x11,
        Key::Shift => 0x10,
        Key::Alt => 0x12,
        Key::Char(ch) if ch.is_ascii_alphanumeric() => ch.to_ascii_uppercase() as u16,
        Key::Char(_) => return None,
    })
}

fn is_extended(vk: u16) -> bool {
    matches!(vk, 0x21..=0x28 | 0x2D | 0x2E)
}

fn virtual_key(vk: u16, up: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if is_extended(vk) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    keyboard(KEYBDINPUT {
        wVk: VIRTUAL_KEY(vk),
        dwFlags: flags,
        ..Default::default()
    })
}

fn unicode(unit: u16, up: bool) -> INPUT {
    let mut flags = KEYEVENTF_UNICODE;
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    keyboard(KEYBDINPUT {
        wVk: VIRTUAL_KEY(0),
        wScan: unit,
        dwFlags: flags,
        ..Default::default()
    })
}

fn keyboard(ki: KEYBDINPUT) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki },
    }
}

fn send(inputs: &[INPUT]) {
    if inputs.is_empty() {
        return;
    }
    unsafe {
        SendInput(inputs, size_of::<INPUT>() as i32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nadra_like() -> Vec<Step> {
        vec![
            Step::Field("reference".into()),
            Step::Key(Key::Tab),
            Step::Setting("contact".into()),
            Step::Key(Key::Space),
        ]
    }

    #[test]
    fn resolves_fields_and_settings() {
        let actions = resolve(
            &nadra_like(),
            |name| (name == "reference").then(|| "04127781229011".to_string()),
            |name| (name == "contact").then(|| "03000000000".to_string()),
        )
        .unwrap();
        assert_eq!(
            actions,
            vec![
                Action::Text("04127781229011".into()),
                Action::Key(Key::Tab),
                Action::Text("03000000000".into()),
                Action::Key(Key::Space),
            ]
        );
    }

    #[test]
    fn missing_setting_is_reported() {
        let result = resolve(&nadra_like(), |_| Some("x".into()), |_| None);
        assert_eq!(result, Err(Missing::Setting("contact".into())));
    }
}
