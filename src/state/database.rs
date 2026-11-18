use gpui_kit::Global;
use surrealdb::{Surreal, engine::local::Db};

pub struct Database(Surreal<Db>);

impl Global for Database {}

impl Database {
    pub fn new(db: Surreal<Db>) -> Self {
        Self(db)
    }
}
