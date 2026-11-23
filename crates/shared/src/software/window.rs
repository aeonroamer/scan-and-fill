use gpui_shared_string::SharedString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowRule {
    pub process: Vec<SharedString>,
    pub title_contains: Option<SharedString>,
}

impl WindowRule {
    pub fn matches(&self, exe_name: &str, title: &str) -> bool {
        let exe_ok = self.process.iter().any(|p| p.eq_ignore_ascii_case(exe_name));
        let title_ok = match &self.title_contains {
            Some(part) => title.to_lowercase().contains(&part.to_lowercase()),
            None => true,
        };
        exe_ok && title_ok
    }
}
