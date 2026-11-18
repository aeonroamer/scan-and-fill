use directories::ProjectDirs;
use gpui_kit::SharedString;
use surrealdb::{
    Surreal,
    engine::local::{Db, SurrealKv},
};

/// Namespace and database every query in the app runs against.
pub const NS: &str = "main";
pub const DB: &str = "main";

/// Opens the on-disk database and selects the app's namespace and database.
///
/// The error is a short, user-facing message ready for the splash screen.
/// The real error is printed to stderr so you still get the details.
pub async fn open() -> Result<Surreal<Db>, SharedString> {
    let dirs = ProjectDirs::from("dev", "AeonLogics", "ScanFill").ok_or_else(|| {
        eprintln!("[db] no home directory found for the data folder");
        SharedString::from("Couldn't find a folder to save your bills.")
    })?;

    let path = dirs.data_dir().join("db");

    let db = Surreal::new::<SurrealKv>(path).await.map_err(|e| {
        eprintln!("[db] open failed: {e:?}");
        SharedString::from("Couldn't open your bills. Restart Scan & Fill to try again.")
    })?;

    db.use_ns(NS).use_db(DB).await.map_err(|e| {
        eprintln!("[db] selecting {NS}/{DB} failed: {e:?}");
        SharedString::from("Couldn't prepare your workspace. Restart Scan & Fill to try again.")
    })?;

    Ok(db)
}
