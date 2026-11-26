use redb::TableDefinition;
use serde::{Deserialize, Serialize};
use shared::Trigger;

use super::store::{self, Table};
use super::{Db, DbError};

const TABLE: Table = TableDefinition::new("settings");
const KEY: &str = "app";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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

pub async fn load_settings(db: &Db) -> Result<Settings, DbError> {
    match store::get(db, TABLE, KEY)? {
        Some(json) => Ok(serde_json::from_str(&json)?),
        None => Ok(Settings::default()),
    }
}

pub async fn save_settings(db: &Db, settings: Settings) -> Result<(), DbError> {
    store::put(db, TABLE, KEY, &serde_json::to_string(&settings)?)?;
    Ok(())
}
