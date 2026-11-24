use gpui_kit::{App, Global};

use super::{Database, Io};
use crate::database::{self, Settings};

pub struct AppSettings(Settings);

impl Global for AppSettings {}

impl AppSettings {
    pub fn provide(settings: Settings, cx: &mut App) {
        cx.set_global(Self(settings));
    }

    pub fn get(cx: &App) -> &Settings {
        &cx.global::<Self>().0
    }

    pub fn set_contact(contact: &str, cx: &mut App) -> bool {
        if !Settings::is_valid_contact(contact) {
            return false;
        }
        let contact = contact.to_string();
        Self::update(cx, |settings| settings.contact = Some(contact));
        true
    }

    pub fn update(cx: &mut App, change: impl FnOnce(&mut Settings)) {
        let mut settings = cx.global::<Self>().0.clone();
        change(&mut settings);
        cx.set_global(Self(settings.clone()));

        let db = Database::get(cx);
        Io::spawn(cx, async move {
            if let Err(e) = database::save_settings(&db, settings).await {
                tracing::error!("saving settings failed: {e}");
            }
        });
    }
}
