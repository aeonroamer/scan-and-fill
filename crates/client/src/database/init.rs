use std::path::PathBuf;
use std::sync::Arc;

use directories::ProjectDirs;
use redb::Database;

use super::{Db, DbError};

const FOLDER: &str = if cfg!(debug_assertions) { "store-dev" } else { "store" };
const FILE: &str = "scanfill.redb";

fn db_path() -> Option<PathBuf> {
    ProjectDirs::from("dev", "AeonLogics", "ScanFill").map(|dirs| dirs.data_local_dir().join(FOLDER))
}

pub async fn open() -> Result<Db, DbError> {
    let path = db_path().ok_or(DbError::NoHome)?;
    std::fs::create_dir_all(&path)?;
    let db = Database::create(path.join(FILE)).map_err(redb::Error::from)?;
    Ok(Arc::new(db))
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
