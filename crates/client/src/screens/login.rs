use std::time::Duration;

use crate::state::Theme;
use gpui_kit::{
    Animation, AnimationExt, App, AppContext, Context, Div, Entity, FontWeight, IntoElement,
    ParentElement, Render, SharedString, Styled, Window, div, ease_in_out, px,
};
use gpui_kit::{
    Subscription,
    component::{
        Sizable,
        button::{Button, ButtonVariants},
        input::{Input, InputContentType, InputEvent, InputState},
    },
};

const APP_NAME: &str = "Scan & Fill";
const COMPANY: &str = "Aeon Logics";
const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

const CARD_W: f32 = 400.0;
const FADE_MS: u64 = 500;

pub struct LoginScreen {
    email: Entity<InputState>,
    password: Entity<InputState>,
    error: Option<SharedString>,
    _subsciptions: Vec<Subscription>,
}

impl LoginScreen {
    pub fn build_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx_self| {
            let email = cx_self
                .new(|cx_self| InputState::new(window, cx_self).placeholder("name@company.com"));
            let password = cx_self.new(|cx_self| {
                InputState::new(window, cx_self)
                    .placeholder("Your password")
                    .masked(true)
            });

            email.update(cx_self, |state, cx_self| state.focus(window, cx_self));

            Self {
                email,
                password,
                error: None,
                _subsciptions: Vec::new(),
            }
        })
    }

    fn submit(&mut self, _window: &mut Window, cx_self: &mut Context<Self>) {
        let email = self.email.read(cx_self).value().trim().to_string();
        let password = self.password.read(cx_self).value();

        let problem = if email.is_empty() {
            Some("Enter your email.")
        } else if !email.contains('@') {
            Some("Enter a valid email address.")
        } else if password.is_empty() {
            Some("Enter your password.")
        } else {
            None
        };

        self.error = problem.map(SharedString::from);
        cx_self.notify();

        if self.error.is_none() {
            tracing::info!("sign in requested for {email}");
        }
    }
}

fn field(label: &'static str, input: Input, theme: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(
            div()
                .text_size(px(13.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text_main)
                .child(label),
        )
        .child(input)
}

impl Render for LoginScreen {
    fn render(&mut self, _window: &mut Window, cx_self: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx_self.global::<Theme>();

        let sign_in = cx_self.listener(|this, _, window, cx_self| this.submit(window, cx_self));

        let card = div()
            .w(px(CARD_W))
            .flex()
            .flex_col()
            .p(px(32.))
            .rounded(px(8.))
            .border_1()
            .border_color(theme.border_subtle)
            .bg(theme.bg_surface)
            .child(div().w(px(32.)).h(px(2.)).rounded_full().bg(theme.accent))
            .child(
                div()
                    .mt(px(20.))
                    .text_size(px(24.))
                    .text_color(theme.text_main)
                    .child("Sign in"),
            )
            .child(
                div()
                    .mt(px(6.))
                    .text_size(px(14.))
                    .text_color(theme.text_muted)
                    .child(format!("Use your {APP_NAME} account to continue.")),
            )
            .child(
                div()
                    .mt(px(28.))
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(field("Email", Input::new(&self.email).large(), &theme))
                    .child(field(
                        "Password",
                        Input::new(&self.password)
                            .large()
                            .content_type(InputContentType::Password)
                            .mask_toggle(),
                        &theme,
                    )),
            )
            .children(self.error.clone().map(|error| {
                div()
                    .mt(px(12.))
                    .text_size(px(13.))
                    .text_color(theme.error)
                    .child(error)
            }))
            .child(
                div().mt(px(24.)).child(
                    Button::new("login-submit")
                        .label("Sign in")
                        .primary()
                        .large()
                        .w_full()
                        .on_click(sign_in),
                ),
            );

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .bg(theme.bg_app)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(card)
                    .with_animation(
                        "login-in",
                        Animation::new(Duration::from_millis(FADE_MS)).with_easing(ease_in_out),
                        |el, delta| el.opacity(delta),
                    ),
            )
            .child(
                div()
                    .pb(px(32.))
                    .flex()
                    .gap(px(8.))
                    .text_size(px(12.))
                    .text_color(theme.text_muted)
                    .child(COMPANY)
                    .child("·")
                    .child(VERSION),
            )
    }
}
