use std::time::Duration;

use gpui_shared_string::SharedString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Tab,
    Enter,
    Space,
    Escape,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Ctrl,
    Shift,
    Alt,
    Char(char),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Field(SharedString),
    Setting(SharedString),
    Text(SharedString),
    Key(Key),
    Combo(Vec<Key>),
    Wait(Duration),
}
