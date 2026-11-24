use gpui_kit::component::{Icon, IconName, Sizable};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    App, ClickEvent, Div, FontWeight, InteractiveElement, ParentElement, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use shared::Software;

use crate::state::{AppSettings, Theme};

pub(crate) fn default_software(softwares: &[Software], cx: &App) -> Option<SharedString> {
    let chosen = AppSettings::get(cx).software.as_deref();
    softwares
        .iter()
        .find(|software| chosen == Some(&*software.id))
        .or(softwares.first())
        .map(|software| software.id.clone())
}

pub(crate) fn card(theme: &Theme) -> Div {
    div()
        .w_full()
        .rounded(px(8.))
        .border_1()
        .border_color(theme.border_subtle)
        .bg(theme.bg_surface)
}

pub(crate) fn picker_card(theme: &Theme) -> Div {
    card(theme).p(px(6.)).flex().flex_col().gap(px(2.))
}

pub(crate) fn pick_row(
    id: SharedString,
    title: SharedString,
    subtitle: SharedString,
    selected: bool,
    theme: &Theme,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover_bg = theme.bg_hover;

    div()
        .id(id)
        .px(px(10.))
        .py(px(8.))
        .rounded(px(6.))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(12.))
        .when(selected, |el| el.bg(theme.bg_selected))
        .when(!selected, |el| el.hover(move |s| s.bg(hover_bg)))
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
                        .when(selected, |el| el.font_weight(FontWeight::MEDIUM))
                        .child(title),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(theme.text_muted)
                        .child(subtitle),
                ),
        )
        .when(selected, |el| {
            el.child(Icon::new(IconName::Check).small().text_color(theme.accent))
        })
        .on_click(on_click)
}

pub(crate) fn rupees(amount: u32) -> String {
    let digits = amount.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    format!("Rs. {out}")
}
