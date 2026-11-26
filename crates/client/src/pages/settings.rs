use gpui_kit::component::{
    IconName,
    button::Button,
    input::{Input, InputEvent, InputState},
    menu::{DropdownMenu, PopupMenuItem},
    switch::Switch,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    App, AppContext, Context, Div, Entity, FontWeight, IntoElement, ParentElement, Render,
    SharedString, Styled, Subscription, Window, div, px,
};
use shared::{Software, SoftwareKind, Trigger};

use super::ui::{card, pick_row, picker_card};
use crate::database::{CONTACT_LEN, Settings};
use crate::state::{AppSettings, Theme};

const MAX_DELAY_MS: u32 = 1000;
const DELAY_DIGITS: usize = 4;

pub struct SettingsPage {
    contact: Entity<InputState>,
    delay: Entity<InputState>,
    softwares: Vec<Software>,
    _subscriptions: Vec<Subscription>,
}

impl SettingsPage {
    pub fn build_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx_self| {
            let settings = AppSettings::get(cx_self).clone();

            let contact = cx_self.new(|cx_self| {
                InputState::new(window, cx_self)
                    .placeholder("03000000000")
                    .default_value(settings.contact().to_string())
                    .validate(|text, _| {
                        text.len() <= CONTACT_LEN && text.chars().all(|c| c.is_ascii_digit())
                    })
            });

            let delay = cx_self.new(|cx_self| {
                let state = InputState::new(window, cx_self)
                    .placeholder("Software default")
                    .validate(|text, _| {
                        text.len() <= DELAY_DIGITS && text.chars().all(|c| c.is_ascii_digit())
                    });
                match settings.step_delay_ms {
                    Some(ms) => state.default_value(ms.to_string()),
                    None => state,
                }
            });

            let subscriptions = vec![
                cx_self.subscribe(
                    &contact,
                    |this: &mut Self, _, event: &InputEvent, cx_self| {
                        if let InputEvent::Change = event {
                            let text = this.contact.read(cx_self).value().to_string();
                            AppSettings::set_contact(&text, cx_self);
                            cx_self.notify();
                        }
                    },
                ),
                cx_self.subscribe(&delay, |this: &mut Self, _, event: &InputEvent, cx_self| {
                    if let InputEvent::Change = event {
                        let text = this.delay.read(cx_self).value().trim().to_string();
                        let value = if text.is_empty() {
                            Some(None)
                        } else {
                            text.parse::<u32>()
                                .ok()
                                .filter(|ms| *ms <= MAX_DELAY_MS)
                                .map(Some)
                        };
                        if let Some(value) = value {
                            AppSettings::update(cx_self, |settings| settings.step_delay_ms = value);
                        }
                        cx_self.notify();
                    }
                }),
                cx_self.observe_global::<AppSettings>(|_: &mut Self, cx_self| cx_self.notify()),
            ];

            Self {
                contact,
                delay,
                softwares: Software::all(),
                _subscriptions: subscriptions,
            }
        })
    }

    fn select_software(id: SharedString, cx: &mut App) {
        AppSettings::update(cx, |settings| settings.software = Some(id.to_string()));
    }
}

fn setting(
    title: &'static str,
    help: &'static str,
    control: impl IntoElement,
    theme: &Theme,
) -> Div {
    div()
        .px(px(20.))
        .py(px(16.))
        .flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap(px(16.))
        .child(
            div()
                .flex_1()
                .min_w(px(260.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text_main)
                        .child(title),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.text_muted)
                        .child(help),
                ),
        )
        .child(div().flex_none().child(control))
}

fn group(title: &'static str, rows: Vec<Div>, theme: &Theme) -> Div {
    let border = theme.border_subtle;
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
        .child(
            card(theme).flex().flex_col().children(
                rows.into_iter().enumerate().map(move |(i, row)| {
                    row.when(i > 0, |el| el.border_t_1().border_color(border))
                }),
            ),
        )
}

impl Render for SettingsPage {
    fn render(&mut self, _window: &mut Window, cx_self: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx_self.global::<Theme>();
        let settings: Settings = AppSettings::get(cx_self).clone();

        let contact_text = self.contact.read(cx_self).value();
        let contact_ok = Settings::is_valid_contact(&contact_text);
        let contact_control = div()
            .w(px(240.))
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(Input::new(&self.contact))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(if contact_ok {
                        theme.text_muted
                    } else {
                        theme.error
                    })
                    .child(if contact_ok {
                        "Saved".to_string()
                    } else {
                        format!(
                            "Must be {CONTACT_LEN} digits ({}/{CONTACT_LEN})",
                            contact_text.len()
                        )
                    }),
            );

        let delay_control = div().w(px(160.)).child(Input::new(&self.delay));

        let current_trigger = settings.trigger();
        let trigger_control = Button::new("settings-trigger")
            .label(current_trigger.label())
            .icon(IconName::ChevronDown)
            .outline()
            .w(px(180.))
            .dropdown_menu(move |menu, _, _| {
                Trigger::all().into_iter().fold(menu, |menu, trigger| {
                    menu.item(
                        PopupMenuItem::new(trigger.label())
                            .checked(trigger == current_trigger)
                            .on_click(move |_, _, cx| {
                                AppSettings::update(cx, |settings| {
                                    settings.trigger = Some(trigger.id())
                                });
                            }),
                    )
                })
            });

        let sound = settings.sound.unwrap_or(true);
        let sound_control =
            Switch::new("settings-sound")
                .checked(sound)
                .on_click(|checked, _, cx| {
                    let checked = *checked;
                    AppSettings::update(cx, |settings| settings.sound = Some(checked));
                });

        let selected = settings.software.clone().or_else(|| {
            self.softwares
                .first()
                .map(|software| software.id.to_string())
        });
        let software_rows: Vec<_> = self
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
                    format!("settings-software-{}", software.id).into(),
                    software.label.clone(),
                    subtitle,
                    selected.as_deref() == Some(&*software.id),
                    &theme,
                    move |_, _, cx| Self::select_software(id.clone(), cx),
                )
            })
            .collect();

        div()
            .w_full()
            .max_w(px(720.))
            .mx_auto()
            .flex()
            .flex_col()
            .gap(px(28.))
            .child(group(
                "Typing",
                vec![
                    setting(
                        "Contact number",
                        "Typed into the portal's contact field. 11 digits.",
                        contact_control,
                        &theme,
                    ),
                    setting(
                        "Typing delay",
                        "Milliseconds between keys. Leave empty for the software's default.",
                        delay_control,
                        &theme,
                    ),
                    setting(
                        "Trigger key",
                        "Bulk bills: types the bill at the cursor. Single bill: continues the form after a reference you typed yourself.",
                        trigger_control,
                        &theme,
                    ),
                    setting(
                        "Sounds",
                        "Play a sound when a barcode isn't recognized.",
                        sound_control,
                        &theme,
                    ),
                ],
                &theme,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(theme.text_muted)
                            .child("Default software"),
                    )
                    .child(picker_card(&theme).children(software_rows)),
            )
    }
}
