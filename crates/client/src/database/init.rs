use directories::ProjectDirs;
use surrealdb::{
    Surreal,
    engine::local::{Db, SurrealKv},
};

use shared::{DB, DbError, NS};

pub async fn open() -> Result<Surreal<Db>, DbError> {
    let dirs = ProjectDirs::from("dev", "AeonLogics", "ScanFill").ok_or(DbError::NoHome)?;
    let path = dirs.data_local_dir().join("db");

    let db = Surreal::new::<SurrealKv>(path)
        .await
        .map_err(DbError::Open)?;
    db.use_ns(NS).use_db(DB).await.map_err(DbError::Namespace)?;

    Ok(db)
}
