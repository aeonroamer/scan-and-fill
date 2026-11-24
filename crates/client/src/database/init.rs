use std::path::PathBuf;

use directories::ProjectDirs;
use surrealdb::{
    Surreal,
    engine::local::{Db, SurrealKv},
};

use super::DbError;
use shared::{DB, NS};

const FOLDER: &str = if cfg!(debug_assertions) { "db-dev" } else { "db" };

fn db_path() -> Option<PathBuf> {
    ProjectDirs::from("dev", "AeonLogics", "ScanFill").map(|dirs| dirs.data_local_dir().join(FOLDER))
}

pub async fn open() -> Result<Surreal<Db>, DbError> {
    let path = db_path().ok_or(DbError::NoHome)?;

    let db = Surreal::new::<SurrealKv>(path)
        .await
        .map_err(DbError::Open)?;
    db.use_ns(NS).use_db(DB).await.map_err(DbError::Namespace)?;

    Ok(db)
}

pub fn reset() -> std::io::Result<Option<PathBuf>> {
    let Some(path) = db_path().filter(|path| path.exists()) else {
        return Ok(None);
    };
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let backup = path.with_file_name(format!("{FOLDER}-broken-{stamp}"));
    std::fs::rename(&path, &backup)?;
    Ok(Some(backup))
}
