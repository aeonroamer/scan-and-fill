use std::time::Duration;

use crate::state::Theme;
use gpui_kit::component::{Sizable, button::Button, spinner::Spinner};
use gpui_kit::{
    Animation, AnimationExt, App, AppContext, Context, Entity, EventEmitter, FontWeight,
    IntoElement, ParentElement, Render, SharedString, Styled, Window, div, ease_in_out, px,
};

const APP_NAME: &str = "Scan & Fill";
const TAGLINE: &str = "Scan the barcode. The form fills itself.";
const COMPANY: &str = "Aeon Logics";

const TITLE_SIZE: f32 = 64.0;
const SLOT_H: f32 = 96.0;
const FADE_MS: u64 = 700;

pub enum SplashEvent {
    Retry,
}

pub struct SplashScreen {
    error: Option<SharedString>,
}

pub trait SplashExt {
    fn set_error<C: AppContext>(&self, cx: &mut C, msg: impl Into<SharedString>);
}

impl SplashExt for Entity<SplashScreen> {
    fn set_error<C: AppContext>(&self, cx: &mut C, msg: impl Into<SharedString>) {
        let msg = msg.into();
        let _ = self.update(cx, |s, cx| s.set_error(msg, cx));
    }
}

impl EventEmitter<SplashEvent> for SplashScreen {}

impl SplashScreen {
    pub fn new() -> Self {
        Self { error: None }
    }

    pub fn build_view(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self::new())
    }

    pub fn set_error(&mut self, error: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.error = Some(error.into());
        cx.notify();
    }
}

impl Render for SplashScreen {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let me = cx.entity().downgrade();

        let slot = match &self.error {
            Some(error) => div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.))
                .child(
                    div()
                        .max_w(px(420.))
                        .text_sm()
                        .text_center()
                        .text_color(theme.error)
                        .child(error.clone()),
                )
                .child(
                    Button::new("splash-retry")
                        .label("Try again")
                        .outline()
                        .small()
                        .on_click(move |_, _, cx| {
                            let _ = me.update(cx, |this, cx| {
                                this.error = None;
                                cx.emit(SplashEvent::Retry);
                                cx.notify();
                            });
                        }),
                )
                .with_animation(
                    "splash-error",
                    Animation::new(Duration::from_millis(300)).with_easing(ease_in_out),
                    |el, delta| el.opacity(delta),
                )
                .into_any_element(),
            None => Spinner::new()
                .small()
                .color(theme.text_muted.into())
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
                            .text_base()
                            .text_color(theme.text_muted)
                            .child(TAGLINE),
                    )
                    .child(
                        div()
                            .mt(px(56.))
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
                    .text_sm()
                    .text_color(theme.text_muted)
                    .child(COMPANY),
            )
    }
}
