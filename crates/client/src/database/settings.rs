use shared::Trigger;
use surrealdb::{Error, Surreal, engine::local::Db, types::SurrealValue};

const TABLE: &str = "settings";
const KEY: &str = "app";

#[derive(Clone, Debug, Default, SurrealValue)]
pub struct Settings {
    pub contact: Option<String>,
    pub software: Option<String>,
    pub step_delay_ms: Option<u32>,
    pub sound: Option<bool>,
    pub trigger: Option<String>,
}

pub const DEFAULT_CONTACT: &str = "03000000000";
pub const CONTACT_LEN: usize = 11;

impl Settings {
    pub fn contact(&self) -> &str {
        self.contact
            .as_deref()
            .filter(|contact| Self::is_valid_contact(contact))
            .unwrap_or(DEFAULT_CONTACT)
    }

    pub fn trigger(&self) -> Trigger {
        self.trigger
            .as_deref()
            .and_then(Trigger::from_id)
            .unwrap_or(Trigger::DEFAULT)
    }

    pub fn is_valid_contact(text: &str) -> bool {
        text.len() == CONTACT_LEN && text.chars().all(|c| c.is_ascii_digit())
    }
}

pub async fn load_settings(db: &Surreal<Db>) -> Result<Settings, Error> {
    let settings: Option<Settings> = db.select((TABLE, KEY)).await?;
    Ok(settings.unwrap_or_default())
}

pub async fn save_settings(db: &Surreal<Db>, settings: Settings) -> Result<(), Error> {
    let _: Option<Settings> = db.upsert((TABLE, KEY)).content(settings).await?;
    Ok(())
}
