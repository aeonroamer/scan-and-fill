use std::rc::Rc;

use gpui_kit::component::{
    Disableable, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    menu::{DropdownMenu, PopupMenuItem},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    App, ClickEvent, Div, FontWeight, InteractiveElement, ParentElement, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use shared::{Software, UTILITIES, utility_label};

use crate::state::{AppSettings, Paying, Theme};

pub(crate) fn paying_row(
    id: &'static str,
    current: Option<Paying>,
    disabled: bool,
    theme: &Theme,
    on_pick: impl Fn(Paying, &mut App) + 'static,
) -> Div {
    let on_pick: Rc<dyn Fn(Paying, &mut App)> = Rc::new(on_pick);
    let label: SharedString = match &current {
        Some(paying) => format!(
            "{} · {}",
            utility_label(&paying.utility).unwrap_or_default(),
            paying.company
        )
        .into(),
        None => "Choose…".into(),
    };

    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(8.))
        .child(
            div()
                .text_size(px(13.))
                .text_color(theme.text_muted)
                .child("Paying for"),
        )
        .child(
            Button::new(id)
                .label(label)
                .icon(IconName::ChevronDown)
                .outline()
                .small()
                .disabled(disabled)
                .dropdown_menu(move |menu, window, cx| {
                    UTILITIES.iter().fold(menu, |menu, utility| {
                        let on_pick = on_pick.clone();
                        let current = current.clone();
                        menu.submenu(utility.label, window, cx, move |sub, _, _| {
                            utility.companies.iter().fold(sub, |sub, company| {
                                let on_pick = on_pick.clone();
                                let paying = Paying {
                                    utility: utility.id.into(),
                                    company: (*company).into(),
                                };
                                let checked = current.as_ref() == Some(&paying);
                                sub.item(
                                    PopupMenuItem::new(*company)
                                        .checked(checked)
                                        .on_click(move |_, _, cx| on_pick(paying.clone(), cx)),
                                )
                            })
                        })
                    })
                }),
        )
}

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
