use std::time::Duration;

use gpui_shared_string::SharedString;

use super::{Key, Step, WindowRule};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoftwareKind {
    Desktop,
    Web,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Software {
    pub id: SharedString,
    pub label: SharedString,
    pub kind: SoftwareKind,
    pub window: WindowRule,
    pub step_delay: Duration,
    pub steps: Vec<Step>,
}

impl Software {
    pub fn all() -> Vec<Self> {
        vec![Self::nadra()]
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
}
