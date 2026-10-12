mod compare;
mod db;
mod domain;
mod extractor;
mod routes;

use std::sync::Arc;

use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

use crate::{extractor::openai::OpenAiExtractor, routes::AppState};

#[tokio::main]
async fn main() {
    // Reads backend/.env if present; real environment variables win.
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let Ok(api_key) = std::env::var("OPENAI_API_KEY") else {
        eprintln!("OPENAI_API_KEY is not set. Put it in backend/.env (see .env.example).");
        std::process::exit(1);
    };
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("DATABASE_URL is not set. Put it in backend/.env (see .env.example).");
        std::process::exit(1);
    };
    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5.4-mini".into());
    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".into());

    let pool = match db::connect(&database_url).await {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("Could not connect to the database: {error}");
            std::process::exit(1);
        }
    };

    let state = AppState { extractor: Arc::new(OpenAiExtractor::new(api_key, model.clone())), pool };

    // Local development only: the frontend runs on another port.
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = routes::router(state).layer(cors);

    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    tracing::info!(%addr, %model, "kyb-backend listening");
    axum::serve(listener, app).await.expect("server");
}
