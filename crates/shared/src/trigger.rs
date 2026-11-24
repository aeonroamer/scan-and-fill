#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trigger {
    Up,
    Down,
    Left,
    Right,
    Shift,
    Ctrl,
    Alt,
}

impl Trigger {
    pub const DEFAULT: Self = Self::Right;

    pub const ALL: [Self; 7] = [
        Self::Right,
        Self::Left,
        Self::Up,
        Self::Down,
        Self::Shift,
        Self::Ctrl,
        Self::Alt,
    ];

    pub fn all() -> Vec<Self> {
        Self::ALL.to_vec()
    }

    pub fn id(self) -> String {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
            Self::Shift => "shift",
            Self::Ctrl => "ctrl",
            Self::Alt => "alt",
        }
        .to_string()
    }

    pub fn label(self) -> String {
        match self {
            Self::Up => "↑  Up arrow",
            Self::Down => "↓  Down arrow",
            Self::Left => "←  Left arrow",
            Self::Right => "→  Right arrow",
            Self::Shift => "Shift",
            Self::Ctrl => "Ctrl",
            Self::Alt => "Alt",
        }
        .to_string()
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|trigger| trigger.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_trigger_round_trips() {
        for trigger in Trigger::all() {
            assert_eq!(Trigger::from_id(&trigger.id()), Some(trigger));
        }
    }
}
