use std::time::Duration;

use super::ui::{card, pick_row, picker_card, rupees};
use crate::state::{Mode, Runner, Theme, alpha};
use gpui_kit::component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    Animation, AnimationExt, AnyElement, App, AppContext, Context, Div, Entity,
    FontWeight, IntoElement, ParentElement, Render, Rgba, SharedString,
    Styled, Subscription, Window, div, ease_in_out, pulsating_between, px,
    relative,
};
use gpui_kit::prelude::FluentBuilder;
use shared::{Bill, BillFormat, Software, SoftwareKind};

const NO_SOFTWARE: &str = "your portal";

const PULSE_MS: u64 = 1600;
const SWEEP_MS: u64 = 1800;

const BARCODE_W: f32 = 0.55;
const BARCODE_MIN_W: f32 = 220.0;
const BARCODE_MAX_W: f32 = 640.0;
const BARCODE_RATIO: f32 = 2.4;
const BEAM_H: f32 = 0.3;
const BARS: [u8; 45] = [
    2, 1, 1, 2, 3, 1, 1, 1, 2, 2, 1, 3, 1, 1, 2, 1, 3, 2, 1, 1, 1, 2, 1, 1, 3, 1, 2, 2, 1, 1, 2, 1,
    1, 3, 1, 2, 1, 1, 2, 3, 1, 1, 2, 1, 2,
];

const SIDE_W: f32 = 320.0;
const STAGE_MIN_H: f32 = 420.0;
const NARROW_W: f32 = 1100.0;

pub struct SingleBillPage {
    runner: Entity<Runner>,
    _subscription: Subscription,
    last: Option<Bill>,
    scanned: u32,
    typed: u32,
    rejected: u32,
    softwares: Vec<Software>,
    formats: Vec<BillFormat>,
    software: Option<SharedString>,
}

impl SingleBillPage {
    pub fn build_view(cx: &mut App) -> Entity<Self> {
        cx.new(|cx_self| {
            let softwares = Software::all();
            let formats = BillFormat::all();
            let runner = Runner::get(cx_self);
            let subscription =
                cx_self.observe(&runner, |_: &mut Self, _, cx_self| cx_self.notify());
            Self {
                runner,
                _subscription: subscription,
                last: None,
                scanned: 0,
                typed: 0,
                rejected: 0,
                software: softwares.first().map(|software| software.id.clone()),
                softwares,
                formats,
            }
        })
    }

    fn scanning(&self, cx: &App) -> bool {
        self.runner.read(cx).mode() == Some(Mode::Single)
    }

    fn toggle_scanning(&mut self, cx_self: &mut Context<Self>) {
        let runner = self.runner.clone();
        if runner.read(cx_self).is_running() {
            runner.update(cx_self, |runner, cx_self| runner.stop(cx_self));
            return;
        }

        let Some(software) = self
            .softwares
            .iter()
            .find(|software| Some(&software.id) == self.software.as_ref())
            .cloned()
        else {
            return;
        };
        runner.update(cx_self, |runner, cx_self| {
            runner.start(Mode::Single, software, None, cx_self)
        });
    }

    fn select_software(&mut self, id: SharedString, cx_self: &mut Context<Self>) {
        if self.software.as_ref() != Some(&id) {
            self.software = Some(id);
            cx_self.notify();
        }
    }

    fn software_label(&self) -> SharedString {
        self.softwares
            .iter()
            .find(|software| Some(&software.id) == self.software.as_ref())
            .map(|software| software.label.clone())
            .unwrap_or_else(|| NO_SOFTWARE.into())
    }

    fn software_picker(&self, theme: &Theme, cx_self: &mut Context<Self>) -> Div {
        let rows: Vec<_> = self
            .softwares
            .iter()
            .map(|software| {
                let id = software.id.clone();
                let subtitle: SharedString = match software.kind {
                    SoftwareKind::Desktop => match software.window.process.first() {
                        Some(exe) => format!("Desktop app · {exe}").into(),
                        None => "Desktop app".into(),
                    },
                    SoftwareKind::Web => "Web portal".into(),
                };
                pick_row(
                    format!("software-{}", software.id).into(),
                    software.label.clone(),
                    subtitle,
                    self.software.as_ref() == Some(&software.id),
                    theme,
                    cx_self.listener(move |this, _, _, cx_self| {
                        this.select_software(id.clone(), cx_self)
                    }),
                )
            })
            .collect();
        picker_card(theme).children(rows)
    }

    fn detected_formats(&self, theme: &Theme) -> Div {
        let rows: Vec<_> = self
            .formats
            .iter()
            .map(|format| {
                div()
                    .px(px(10.))
                    .py(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .text_color(theme.text_main)
                                    .child(format.label.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(theme.text_muted)
                                    .child(format!("{}-digit barcode", format.length)),
                            ),
                    )
                    .child(
                        Icon::new(IconName::CircleCheck)
                            .small()
                            .text_color(theme.success),
                    )
            })
            .collect();

        picker_card(theme).children(rows).child(
            div()
                .px(px(10.))
                .pt(px(4.))
                .pb(px(6.))
                .text_size(px(12.))
                .text_color(theme.text_muted)
                .child("The bill type is detected from the barcode."),
        )
    }

    fn stage(&self, theme: &Theme, cx_self: &mut Context<Self>) -> Div {
        let target = self.software_label();
        let scanning = self.scanning(cx_self);
        let (title, helper) = if scanning {
            (
                "Listening for barcodes",
                format!("Each bill is typed into {target} as soon as it's scanned."),
            )
        } else {
            (
                "Ready when you are",
                "Start scanning, then point your scanner at a bill's barcode.".to_string(),
            )
        };

        let toggle = cx_self.listener(|this, _, _, cx_self| this.toggle_scanning(cx_self));
        let button = if scanning {
            Button::new("scan-toggle")
                .label("Stop scanning")
                .icon(IconName::Square)
                .outline()
        } else {
            Button::new("scan-toggle")
                .label("Start scanning")
                .icon(IconName::Play)
                .primary()
        };

        card(theme)
            .flex_1()
            .min_w_0()
            .min_h(px(STAGE_MIN_H))
            .px(px(32.))
            .py(px(40.))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(barcode(scanning, theme))
            .child(
                div()
                    .mt(px(32.))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(status_dot(scanning, theme))
                    .child(
                        div()
                            .text_size(px(20.))
                            .text_color(theme.text_main)
                            .child(title),
                    ),
            )
            .child(
                div()
                    .mt(px(6.))
                    .max_w(px(380.))
                    .text_center()
                    .text_size(px(14.))
                    .text_color(theme.text_muted)
                    .child(helper),
            )
            .child(div().mt(px(24.)).child(button.large().on_click(toggle)))
            .child(
                div()
                    .mt(px(20.))
                    .px(px(10.))
                    .py(px(3.))
                    .rounded_full()
                    .border_1()
                    .border_color(theme.border_subtle)
                    .bg(theme.bg_panel)
                    .text_size(px(12.))
                    .text_color(theme.text_muted)
                    .child(format!("Types into {target}")),
            )
    }

    fn last_scanned(&self, theme: &Theme, mono: SharedString) -> Div {
        let Some(bill) = &self.last else {
            return card(theme)
                .p(px(24.))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.))
                .child(
                    Icon::new(IconName::FileText)
                        .large()
                        .text_color(theme.text_disabled),
                )
                .child(
                    div()
                        .mt(px(8.))
                        .text_size(px(14.))
                        .text_color(theme.text_main)
                        .child("No bill scanned yet"),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.text_muted)
                        .child("Your last bill shows up here."),
                );
        };

        let (badge_text, badge_fg, badge_bg) = if bill.late_fee {
            ("Late fee", theme.warning, theme.warning_subtle)
        } else {
            ("On time", theme.success, theme.success_subtle)
        };

        card(theme)
            .p(px(20.))
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(theme.text_muted)
                            .child(bill.label.clone()),
                    )
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(2.))
                            .rounded_full()
                            .bg(badge_bg)
                            .text_size(px(12.))
                            .text_color(badge_fg)
                            .child(badge_text),
                    ),
            )
            .child(
                div()
                    .mt(px(8.))
                    .text_size(px(28.))
                    .text_color(theme.text_main)
                    .child(rupees(bill.amount)),
            )
            .child(divider(theme).my(px(16.)))
            .child(row(
                "Reference",
                div()
                    .font_family(mono)
                    .text_size(px(14.))
                    .text_color(theme.text_main)
                    .child(bill.reference.clone()),
                theme,
            ))
    }

    fn session(&self, theme: &Theme) -> Div {
        let stat = |label: &'static str, value: u32, color: Rgba| {
            row(
                label,
                div()
                    .text_size(px(14.))
                    .text_color(color)
                    .child(value.to_string()),
                theme,
            )
        };

        let rejected_color = if self.rejected > 0 {
            theme.warning
        } else {
            theme.text_main
        };

        card(theme)
            .p(px(20.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(stat("Scanned", self.scanned, theme.text_main))
            .child(stat("Typed into portal", self.typed, theme.text_main))
            .child(stat("Not recognized", self.rejected, rejected_color))
    }
}

fn barcode(scanning: bool, theme: &Theme) -> Div {
    let bar_color = if scanning {
        theme.text_main
    } else {
        theme.border_strong
    };
    let total: f32 = BARS.iter().map(|w| *w as f32).sum();

    let bars = div().size_full().flex().children(BARS.iter().enumerate().map(|(i, w)| {
        let bar = div().h_full().w(relative(*w as f32 / total));
        if i % 2 == 0 { bar.bg(bar_color) } else { bar }
    }));

    let frame = div()
        .relative()
        .w(relative(BARCODE_W))
        .min_w(px(BARCODE_MIN_W))
        .max_w(px(BARCODE_MAX_W))
        .aspect_ratio(BARCODE_RATIO)
        .overflow_hidden()
        .child(bars);

    if !scanning {
        return frame;
    }

    let beam = div()
        .absolute()
        .left_0()
        .w_full()
        .h(relative(BEAM_H))
        .flex()
        .flex_col()
        .child(div().flex_1().bg(alpha(theme.accent, 0.12)))
        .child(div().flex_none().h(px(2.)).bg(theme.accent))
        .with_animation(
            "scan-sweep",
            Animation::new(Duration::from_millis(SWEEP_MS))
                .repeat()
                .with_easing(ease_in_out),
            |el, delta| el.top(relative(-BEAM_H + delta * (1.0 + BEAM_H))),
        );

    frame.child(beam)
}

fn status_dot(scanning: bool, theme: &Theme) -> AnyElement {
    let dot = div().size(px(8.)).rounded_full();
    if scanning {
        dot.bg(theme.accent)
            .with_animation(
                "scan-pulse",
                Animation::new(Duration::from_millis(PULSE_MS))
                    .repeat()
                    .with_easing(pulsating_between(0.35, 1.0)),
                |el, delta| el.opacity(delta),
            )
            .into_any_element()
    } else {
        dot.bg(theme.text_disabled).into_any_element()
    }
}

fn divider(theme: &Theme) -> Div {
    div().h(px(1.)).bg(theme.border_subtle)
}

fn row(label: &'static str, value: Div, theme: &Theme) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(13.))
                .text_color(theme.text_muted)
                .child(label),
        )
        .child(value)
}

fn section(title: &'static str, body: Div, theme: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .text_size(px(13.))
                .text_color(theme.text_muted)
                .child(title),
        )
        .child(body)
}

impl Render for SingleBillPage {
    fn render(&mut self, window: &mut Window, cx_self: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx_self.global::<Theme>();
        let mono = cx_self.theme().mono_font_family.clone();
        let narrow = window.viewport_size().width < px(NARROW_W);

        let software = section("Software", self.software_picker(&theme, cx_self), &theme);
        let format = section("Recognized bills", self.detected_formats(&theme), &theme);
        let last = section("Last scanned", self.last_scanned(&theme, mono), &theme);
        let session = section("This session", self.session(&theme), &theme);

        let side = if narrow {
            let pair = |a: Div, b: Div| {
                div()
                    .w_full()
                    .flex()
                    .gap(px(24.))
                    .child(a.flex_1().min_w_0())
                    .child(b.flex_1().min_w_0())
            };
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(24.))
                .child(pair(software, format))
                .child(pair(last, session))
        } else {
            div()
                .w(px(SIDE_W))
                .flex_none()
                .flex()
                .flex_col()
                .gap(px(24.))
                .child(software)
                .child(format)
                .child(last)
                .child(session)
        };

        div()
            .size_full()
            .flex()
            .when(narrow, |el| el.flex_col())
            .gap(px(24.))
            .child(self.stage(&theme, cx_self))
            .child(side)
    }
}
