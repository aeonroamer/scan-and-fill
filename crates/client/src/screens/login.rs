use crate::state::Theme;
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};

/// Placeholder for the sign-in step (`Screen::Auth`).
pub struct LoginScreen;

impl LoginScreen {
    pub fn build_view(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self)
    }
}

impl Render for LoginScreen {
    fn render(&mut self, _window: &mut Window, cx_self: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx_self.global::<Theme>();

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.bg_app)
            .text_color(theme.text_muted)
            .child("Auth Screen")
    }
}
