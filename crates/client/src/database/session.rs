use surrealdb::{Error, Surreal, engine::local::Db, types::SurrealValue};

#[derive(SurrealValue)]
pub struct Session {
    token: String,
}

impl Session {
    pub async fn get_session(db: Surreal<Db>) -> Result<Option<Session>, Error> {
        let ses: Option<Session> = db.select(("app", "session")).await.map_err(|e| {
            tracing::error!("reading session failed: {e}");
            e
        })?;
        Ok(ses)
    }
}
