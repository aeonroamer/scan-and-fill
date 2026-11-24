use surrealdb::{Error, Surreal, engine::local::Db, types::SurrealValue};

const TABLE: &str = "bulk_session";

#[derive(Clone, Debug, SurrealValue)]
pub struct StoredBill {
    pub reference: String,
    pub source: String,
    pub format: Option<String>,
    pub label: Option<String>,
    pub late_fee: bool,
    pub original_amount: Option<u32>,
    pub amount: Option<u32>,
    pub paid: Option<bool>,
}

#[derive(Clone, Debug, SurrealValue)]
pub struct BulkSession {
    pub key: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub cursor: Option<u32>,
    pub bills: Vec<StoredBill>,
}

pub async fn list_sessions(db: &Surreal<Db>) -> Result<Vec<BulkSession>, Error> {
    let mut sessions: Vec<BulkSession> = db.select(TABLE).await?;
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(sessions)
}

pub async fn save_session(db: &Surreal<Db>, session: BulkSession) -> Result<(), Error> {
    let _: Option<BulkSession> = db
        .upsert((TABLE, session.key.clone()))
        .content(session)
        .await?;
    Ok(())
}

pub async fn delete_session(db: &Surreal<Db>, key: String) -> Result<(), Error> {
    let _: Option<BulkSession> = db.delete((TABLE, key)).await?;
    Ok(())
}
