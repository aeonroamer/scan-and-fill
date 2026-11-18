use crate::state::Theme;
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};

/// Placeholder for the main app (`Screen::Dashboard`).
/// Will own `Route` and the sidebar later.
pub struct DashboardScreen;

impl DashboardScreen {
    pub fn build_view(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self)
    }
}

impl Render for DashboardScreen {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.bg_app)
            .text_color(theme.text_muted)
            .child("Dashboard Screen")
    }
}
