use gpui_kit::{App, Global};

use crate::database::Db;

pub struct Database(Db);

impl Global for Database {}

impl Database {
    pub fn new(db: Db) -> Self {
        Self(db)
    }

    pub fn get(cx: &App) -> Db {
        cx.global::<Self>().0.clone()
    }
}
