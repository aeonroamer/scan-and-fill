mod app;
mod state;
mod template;

use app::AppServer;
use axum::Router;
use tracing_subscriber::EnvFilter;

const DEFAULT_FILTER: &str = "warn,scanfill_server=info,shared=info";

#[tokio::main]
async fn main() {
    init_tracing();
    tracing::info!("starting Scan & Fill server {}", env!("CARGO_PKG_VERSION"));
    serve(AppServer::router()).await;
}

fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);

    #[cfg(feature = "lambda")]
    let builder = builder.with_ansi(false).without_time();

    builder.init();
}

#[cfg(feature = "lambda")]
async fn serve(app: Router) {
    tracing::info!("running as an AWS Lambda handler");
    if let Err(e) = lambda_http::run(app).await {
        tracing::error!("lambda runtime stopped: {e}");
    }
}

#[cfg(not(feature = "lambda"))]
async fn serve(app: Router) {
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:3000").await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("couldn't bind 127.0.0.1:3000: {e}");
            return;
        }
    };

    tracing::info!("listening on http://127.0.0.1:3000");
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("server stopped: {e}");
    }
}
