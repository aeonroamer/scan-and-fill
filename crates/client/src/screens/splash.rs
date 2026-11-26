use std::time::Duration;

use crate::state::Theme;
use gpui_kit::component::{
    Sizable,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    Animation, AnimationExt, App, AppContext, Context, Entity, EventEmitter, FontWeight,
    IntoElement, ParentElement, Render, SharedString, Styled, Window, div, ease_in_out, px,
};

const APP_NAME: &str = "Scan & Fill";
const TAGLINE: &str = "Scan the barcode. The form fills itself.";
const STATUS: &str = "Getting your workspace ready…";
const COMPANY: &str = "Aeon Logics";
const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));
const RESET_HINT: &str = "Reset starts fresh. Your old data is kept as a backup.";

const TITLE_SIZE: f32 = 56.0;
const SLOT_H: f32 = 120.0;
const FADE_MS: u64 = 700;

const TRACK_W: f32 = 160.0;
const BEAM_W: f32 = 48.0;
const BEAM_MS: u64 = 1400;

pub enum SplashEvent {
    Retry,
    Reset,
}

pub struct SplashScreen {
    error: Option<SharedString>,
    detail: Option<SharedString>,
}
impl EventEmitter<SplashEvent> for SplashScreen {}

impl SplashScreen {
    pub fn new() -> Self {
        Self {
            error: None,
            detail: None,
        }
    }

    pub fn build_view(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self::new())
    }

    pub fn set_error(&mut self, error: impl Into<SharedString>) {
        self.error = Some(error.into());
    }

    pub fn set_detail(&mut self, detail: impl Into<SharedString>) {
        self.detail = Some(detail.into());
    }
}

impl Render for SplashScreen {
    fn render(&mut self, _window: &mut Window, cx_self: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx_self.global::<Theme>();

        let retry_event = cx_self.listener(|this, _, _, cx_self| {
            this.error = None;
            this.detail = None;
            cx_self.emit(SplashEvent::Retry);
            cx_self.notify();
        });
        let reset_event = cx_self.listener(|this, _, _, cx_self| {
            this.error = None;
            this.detail = None;
            cx_self.emit(SplashEvent::Reset);
            cx_self.notify();
        });

        let slot = match &self.error {
            Some(error) => div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.))
                .child(
                    div()
                        .max_w(px(420.))
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.))
                        .text_center()
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(theme.error)
                                .child(error.clone()),
                        )
                        .children(self.detail.clone().map(|detail| {
                            div()
                                .text_size(px(12.))
                                .text_color(theme.text_muted)
                                .child(detail)
                        })),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .child(
                            Button::new("splash-retry")
                                .label("Try again")
                                .outline()
                                .small()
                                .on_click(retry_event),
                        )
                        .child(
                            Button::new("splash-reset")
                                .label("Reset data")
                                .ghost()
                                .small()
                                .on_click(reset_event),
                        ),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(theme.text_muted)
                        .child(RESET_HINT),
                )
                .with_animation(
                    "splash-error",
                    Animation::new(Duration::from_millis(300)).with_easing(ease_in_out),
                    |el, delta| el.opacity(delta),
                )
                .into_any_element(),
            None => div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.))
                .child(
                    div()
                        .relative()
                        .w(px(TRACK_W))
                        .h(px(2.))
                        .rounded_full()
                        .overflow_hidden()
                        .bg(theme.border)
                        .child(
                            div()
                                .absolute()
                                .top_0()
                                .h_full()
                                .w(px(BEAM_W))
                                .rounded_full()
                                .bg(theme.accent)
                                .with_animation(
                                    "splash-beam",
                                    Animation::new(Duration::from_millis(BEAM_MS))
                                        .repeat()
                                        .with_easing(ease_in_out),
                                    |el, delta| el.left(px(-BEAM_W + delta * (TRACK_W + BEAM_W))),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.text_muted)
                        .child(STATUS),
                )
                .into_any_element(),
        };

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
                    .child(
                        div()
                            .text_size(px(TITLE_SIZE))
                            .font_weight(FontWeight::LIGHT)
                            .text_color(theme.text_main)
                            .child(APP_NAME),
                    )
                    .child(
                        div()
                            .mt(px(12.))
                            .text_size(px(15.))
                            .text_color(theme.text_muted)
                            .child(TAGLINE),
                    )
                    .child(
                        div()
                            .mt(px(48.))
                            .h(px(SLOT_H))
                            .flex()
                            .items_start()
                            .justify_center()
                            .child(slot),
                    )
                    .with_animation(
                        "splash-in",
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
