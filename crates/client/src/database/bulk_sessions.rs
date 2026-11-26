use redb::TableDefinition;
use serde::{Deserialize, Serialize};

use super::store::{self, Table};
use super::{Db, DbError};

const TABLE: Table = TableDefinition::new("bulk_session");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredBill {
    pub reference: String,
    pub source: String,
    pub format: Option<String>,
    pub label: Option<String>,
    pub company: Option<String>,
    pub late_fee: bool,
    pub original_amount: Option<u32>,
    pub amount: Option<u32>,
    pub paid: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BulkSession {
    pub key: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub cursor: Option<u32>,
    pub bills: Vec<StoredBill>,
}

pub async fn list_sessions(db: &Db) -> Result<Vec<BulkSession>, DbError> {
    let mut sessions: Vec<BulkSession> = store::all(db, TABLE)?
        .iter()
        .filter_map(|json| match serde_json::from_str(json) {
            Ok(session) => Some(session),
            Err(e) => {
                tracing::warn!("skipping an unreadable bulk session: {e}");
                None
            }
        })
        .collect();
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(sessions)
}

pub async fn save_session(db: &Db, session: BulkSession) -> Result<(), DbError> {
    store::put(db, TABLE, &session.key, &serde_json::to_string(&session)?)?;
    Ok(())
}

pub async fn delete_session(db: &Db, key: String) -> Result<(), DbError> {
    store::remove(db, TABLE, &key)?;
    Ok(())
}
