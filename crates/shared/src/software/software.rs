use std::time::Duration;

use gpui_shared_string::SharedString;

use super::{Key, Step, WindowRule};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoftwareKind {
    Desktop,
    Web,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pick {
    pub field: SharedString,
    pub value: SharedString,
    pub keys: SharedString,
}

impl Pick {
    fn new(field: &str, value: &str, keys: &str) -> Self {
        Self {
            field: field.to_string().into(),
            value: value.to_string().into(),
            keys: keys.to_string().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Software {
    pub id: SharedString,
    pub label: SharedString,
    pub kind: SoftwareKind,
    pub window: WindowRule,
    pub step_delay: Duration,
    pub steps: Vec<Step>,
    pub picks: Vec<Pick>,
    pub start_hint: SharedString,
    pub max_batch: usize,
}

impl Software {
    pub fn all() -> Vec<Self> {
        vec![Self::nadra(), Self::easypaisa()]
    }

    pub fn needs_company(&self) -> bool {
        self.picks.iter().any(|pick| pick.field == "company")
    }

    pub fn pick(&self, field: &str, value: &str) -> Option<SharedString> {
        self.picks
            .iter()
            .find(|pick| pick.field == field && pick.value.eq_ignore_ascii_case(value))
            .map(|pick| pick.keys.clone())
    }

    pub fn easypaisa() -> Self {
        Self {
            id: "easypaisa".into(),
            label: "Easypaisa".into(),
            kind: SoftwareKind::Web,
            window: WindowRule {
                process: vec!["firefox.exe".into(), "chrome.exe".into(), "msedge.exe".into()],
                title_contains: Some("Bulk Payment Bill Payment".into()),
            },
            step_delay: Duration::from_millis(5),
            steps: vec![
                Step::Combo(vec![Key::Shift, Key::Tab]),
                Step::Combo(vec![Key::Shift, Key::Tab]),
                Step::Combo(vec![Key::Shift, Key::Tab]),
                Step::Key(Key::Space),
                Step::Key(Key::Tab),
                Step::Key(Key::Home),
                Step::Field("utility".into()),
                Step::Wait(Duration::from_millis(250)),
                Step::Key(Key::Tab),
                Step::Key(Key::Home),
                Step::Field("company".into()),
                Step::Key(Key::Tab),
                Step::Field("reference".into()),
                Step::Key(Key::Tab),
                Step::Setting("contact".into()),
                Step::Key(Key::Tab),
                Step::Key(Key::Tab),
                Step::Key(Key::Tab),
                Step::Key(Key::Tab),
                Step::Wait(Duration::from_millis(300)),
            ],
            start_hint: "Click row 1's Consumer Number box. Utility and company are picked for you.".into(),
            max_batch: 80,
            picks: [
                ("utility", "electricity", "E"),
                ("utility", "gas", "G"),
                ("company", "FESCO", "F"),
                ("company", "GEPCO", "G"),
                ("company", "HESCO", "H"),
                ("company", "IESCO", "I"),
                ("company", "LESCO", "L"),
                ("company", "MEPCO", "M"),
                ("company", "PESCO", "P"),
                ("company", "QESCO", "Q"),
                ("company", "HAZECO", "HH"),
                ("company", "SNGPL", "S"),
                ("company", "SSGC", "SS"),
            ]
            .into_iter()
            .map(|(field, value, keys)| Pick::new(field, value, keys))
            .collect(),
        }
    }

    pub fn find(id: &str) -> Option<Self> {
        Self::all().into_iter().find(|software| software.id == id)
    }

    pub fn nadra() -> Self {
        Self {
            id: "nadra".into(),
            label: "Nadra".into(),
            kind: SoftwareKind::Desktop,
            window: WindowRule {
                process: vec!["e-Sahulat.exe".into()],
                title_contains: None,
            },
            step_delay: Duration::from_millis(1),
            steps: vec![
                Step::Field("reference".into()),
                Step::Key(Key::Tab),
                Step::Key(Key::Tab),
                Step::Key(Key::Tab),
                Step::Setting("contact".into()),
                Step::Key(Key::Tab),
                Step::Key(Key::Space),
                Step::Key(Key::Tab),
                Step::Key(Key::Tab),
                Step::Key(Key::Space),
            ],
            picks: Vec::new(),
            start_hint: "Click the reference number field in e-Sahulat first.".into(),
            max_batch: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn ids_are_unique_and_not_empty() {
        let all = Software::all();
        let ids: HashSet<_> = all.iter().map(|software| software.id.clone()).collect();
        assert_eq!(ids.len(), all.len());
        assert!(all.iter().all(|software| !software.id.is_empty()));
    }

    #[test]
    fn every_software_has_a_window_and_steps() {
        for software in Software::all() {
            assert!(!software.window.process.is_empty(), "{} has no process", software.id);
            assert!(!software.steps.is_empty(), "{} has no steps", software.id);
        }
    }

    #[test]
    fn nadra_matches_e_sahulat_only() {
        let rule = Software::nadra().window;
        assert!(rule.matches("e-Sahulat.exe", "Login"));
        assert!(rule.matches("E-SAHULAT.EXE", "Bill Payment"));
        assert!(!rule.matches("chrome.exe", "e-Sahulat"));
    }

    #[test]
    fn easypaisa_matches_its_tab_in_any_browser() {
        let rule = Software::easypaisa().window;
        assert!(rule.matches("firefox.exe", "Bulk Payment Bill Payment — Mozilla Firefox"));
        assert!(rule.matches("chrome.exe", "Bulk Payment Bill Payment - Google Chrome"));
        assert!(!rule.matches("firefox.exe", "YouTube — Mozilla Firefox"));
    }

    #[test]
    fn easypaisa_picks_keys_for_companies() {
        let easypaisa = Software::easypaisa();
        assert_eq!(easypaisa.pick("utility", "electricity").as_deref(), Some("E"));
        assert_eq!(easypaisa.pick("company", "MEPCO").as_deref(), Some("M"));
        assert_eq!(easypaisa.pick("company", "SSGC").as_deref(), Some("SS"));
        assert_eq!(easypaisa.pick("company", "TESCO"), None);
    }
}
