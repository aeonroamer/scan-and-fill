use surrealdb::{Surreal, engine::any::Any};

pub struct AppState {
    db_conn: Surreal<Any>,
}
