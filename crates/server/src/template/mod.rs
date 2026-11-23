use axum::response::Html;
pub struct PageRenderer;

impl PageRenderer {
    pub fn home() -> Html<&'static str> {
        Html(include_str!("./index.html"))
    }
}
