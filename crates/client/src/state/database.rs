use gpui_kit::{App, Global};
use surrealdb::{Surreal, engine::local::Db};

pub struct Database(Surreal<Db>);

impl Global for Database {}

impl Database {
    pub fn new(db: Surreal<Db>) -> Self {
        Self(db)
    }

    pub fn get(cx: &App) -> Surreal<Db> {
        cx.global::<Self>().0.clone()
    }
}
