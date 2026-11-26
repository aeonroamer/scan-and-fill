use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use shared::{Trigger, WindowRule};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, EVENT_OBJECT_NAMECHANGE, EVENT_SYSTEM_FOREGROUND,
    GetForegroundWindow, GetMessageW, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, PM_NOREMOVE,
    PeekMessageW, PostThreadMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    WH_KEYBOARD_LL, WINEVENT_OUTOFCONTEXT, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN,
    WM_SYSKEYUP,
};

use crate::{KeyInput, focused_window};

const VK_ENTER: u32 = 0x0D;
const VK_CTRL: i32 = 0x11;
const VK_ALT: i32 = 0x12;
const OBJID_WINDOW: i32 = 0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HookEvent {
    Key(KeyInput, Instant),
    Trigger,
    Focus(bool),
}

static ACTIVE: AtomicBool = AtomicBool::new(false);
static BUFFERED: AtomicUsize = AtomicUsize::new(0);
static ENTER_SWALLOWED: AtomicBool = AtomicBool::new(false);
static TAP_PENDING: AtomicU32 = AtomicU32::new(0);
static SENDER: Mutex<Option<Sender<HookEvent>>> = Mutex::new(None);
static RULE: Mutex<Option<WindowRule>> = Mutex::new(None);
static TRIGGER: Mutex<Option<Trigger>> = Mutex::new(None);

pub struct Hook {
    thread_id: u32,
    handle: Option<JoinHandle<()>>,
}

impl Hook {
    pub fn start(
        rule: WindowRule,
        trigger: Option<Trigger>,
        sender: Sender<HookEvent>,
    ) -> Option<Self> {
        let (ready_tx, ready_rx) = mpsc::channel();

        let handle = thread::Builder::new()
            .name("key-hook".into())
            .spawn(move || run(rule, trigger, sender, ready_tx))
            .ok()?;

        match ready_rx.recv() {
            Ok(Some(thread_id)) => Some(Self {
                thread_id,
                handle: Some(handle),
            }),
            _ => {
                let _ = handle.join();
                None
            }
        }
    }

    pub fn clear_buffered() {
        BUFFERED.store(0, Ordering::Relaxed);
    }

    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        let Some(handle) = self.handle.take() else {
            return;
        };
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        let _ = handle.join();
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn run(
    rule: WindowRule,
    trigger: Option<Trigger>,
    sender: Sender<HookEvent>,
    ready: Sender<Option<u32>>,
) {
    *SENDER.lock().unwrap() = Some(sender);
    *RULE.lock().unwrap() = Some(rule);
    *TRIGGER.lock().unwrap() = trigger;
    BUFFERED.store(0, Ordering::Relaxed);
    ACTIVE.store(false, Ordering::Relaxed);

    let mut msg = MSG::default();
    unsafe {
        let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);
    }

    let keyboard = unsafe {
        GetModuleHandleW(None).ok().and_then(|module| {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_proc),
                Some(HINSTANCE(module.0)),
                0,
            )
            .ok()
        })
    };

    let Some(keyboard) = keyboard else {
        let _ = ready.send(None);
        clear_state();
        return;
    };

    let foreground = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(window_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    let renamed = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_NAMECHANGE,
            EVENT_OBJECT_NAMECHANGE,
            None,
            Some(window_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };

    let _ = ready.send(Some(unsafe { GetCurrentThreadId() }));
    refresh_focus();

    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnhookWindowsHookEx(keyboard);
        let _ = UnhookWinEvent(foreground);
        let _ = UnhookWinEvent(renamed);
    }
    clear_state();
}

fn clear_state() {
    ACTIVE.store(false, Ordering::Relaxed);
    BUFFERED.store(0, Ordering::Relaxed);
    *SENDER.lock().unwrap() = None;
    *RULE.lock().unwrap() = None;
    *TRIGGER.lock().unwrap() = None;
}

fn send(event: HookEvent) -> bool {
    match SENDER.lock() {
        Ok(guard) => guard.as_ref().is_some_and(|tx| tx.send(event).is_ok()),
        Err(_) => false,
    }
}

fn refresh_focus() {
    let active = match (
        focused_window(),
        RULE.lock().ok().and_then(|rule| rule.clone()),
    ) {
        (Some(info), Some(rule)) => info
            .exe_name()
            .is_some_and(|exe| rule.matches(exe, &info.title)),
        _ => false,
    };

    if ACTIVE.swap(active, Ordering::Relaxed) != active {
        BUFFERED.store(0, Ordering::Relaxed);
        send(HookEvent::Focus(active));
    }
}

unsafe extern "system" fn window_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if event == EVENT_OBJECT_NAMECHANGE
        && (id_object != OBJID_WINDOW || hwnd != unsafe { GetForegroundWindow() })
    {
        return;
    }
    refresh_focus();
}

fn trigger_vk(trigger: Trigger) -> &'static [u32] {
    match trigger {
        Trigger::Up => &[0x26],
        Trigger::Down => &[0x28],
        Trigger::Left => &[0x25],
        Trigger::Right => &[0x27],
        Trigger::Shift => &[0x10, 0xA0, 0xA1],
        Trigger::Ctrl => &[0x11, 0xA2, 0xA3],
        Trigger::Alt => &[0x12, 0xA4, 0xA5],
    }
}

fn is_modifier(trigger: Trigger) -> bool {
    matches!(trigger, Trigger::Shift | Trigger::Ctrl | Trigger::Alt)
}

fn captured_char(vk: u32) -> Option<char> {
    match vk {
        0x30..=0x39 => char::from_digit(vk - 0x30, 10),
        0x60..=0x69 => char::from_digit(vk - 0x60, 10),
        0x41..=0x5A => char::from_u32(vk),
        _ => None,
    }
}

fn held(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

enum Verdict {
    Pass,
    Swallow,
}

fn handle_key(vk: u32, down: bool) -> Verdict {
    let trigger = TRIGGER.lock().ok().and_then(|trigger| *trigger);

    if let Some(trigger) = trigger {
        let keys = trigger_vk(trigger);
        if is_modifier(trigger) {
            if keys.contains(&vk) {
                if down {
                    TAP_PENDING.store(vk, Ordering::Relaxed);
                } else if TAP_PENDING.swap(0, Ordering::Relaxed) == vk {
                    send(HookEvent::Trigger);
                }
                return Verdict::Pass;
            }
            if down {
                TAP_PENDING.store(0, Ordering::Relaxed);
            }
        } else if keys.contains(&vk) {
            if down {
                send(HookEvent::Trigger);
            }
            return Verdict::Swallow;
        }
    }

    if held(VK_CTRL) || held(VK_ALT) {
        return Verdict::Pass;
    }

    if vk == VK_ENTER {
        if down {
            if BUFFERED.swap(0, Ordering::Relaxed) == 0 {
                ENTER_SWALLOWED.store(false, Ordering::Relaxed);
                return Verdict::Pass;
            }
            ENTER_SWALLOWED.store(true, Ordering::Relaxed);
            send(HookEvent::Key(KeyInput::Enter, Instant::now()));
            return Verdict::Swallow;
        }
        return if ENTER_SWALLOWED.swap(false, Ordering::Relaxed) {
            Verdict::Swallow
        } else {
            Verdict::Pass
        };
    }

    let Some(ch) = captured_char(vk) else {
        return Verdict::Pass;
    };
    if down {
        if !send(HookEvent::Key(KeyInput::Char(ch), Instant::now())) {
            return Verdict::Pass;
        }
        BUFFERED.fetch_add(1, Ordering::Relaxed);
    }
    Verdict::Swallow
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && ACTIVE.load(Ordering::Relaxed) {
        let info = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let injected = (info.flags.0 & LLKHF_INJECTED.0) != 0;
        let message = wparam.0 as u32;
        let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
        let up = message == WM_KEYUP || message == WM_SYSKEYUP;

        if !injected && (down || up) {
            if let Verdict::Swallow = handle_key(info.vkCode, down) {
                return LRESULT(1);
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
