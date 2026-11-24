use gpui_kit::{App, Global};

#[derive(Default)]
pub struct RunState {
    running: bool,
}

impl Global for RunState {}

impl RunState {
    pub fn is_running(cx: &App) -> bool {
        cx.try_global::<Self>().is_some_and(|state| state.running)
    }

    pub fn set(running: bool, cx: &mut App) {
        cx.set_global(Self { running });
    }
}
