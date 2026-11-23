use crate::template::PageRenderer;
use axum::{Router, routing::get};

pub struct AppServer;

impl AppServer {
    pub fn router() -> Router {
        let router = Router::new().route("/", get(PageRenderer::home()));

        router
    }
}
