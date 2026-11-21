use axum::{Router, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Scan & Fill server" }));

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("couldn't bind 127.0.0.1:3000");

    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.expect("server stopped");
}
