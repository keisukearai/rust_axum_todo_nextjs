pub mod config;
pub mod error;
pub mod state;
pub mod todo;

use axum::Router;
use axum::http::header::InvalidHeaderValue;
use axum::http::{HeaderValue, Method, header};
use axum::routing::get;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use config::Config;
use state::AppState;

/// 別プロセスで動く PostgreSQL への接続プールを作り、マイグレーションを当てる。
pub async fn connect_database(config: &Config) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("../db/migrations").run(&pool).await?;

    Ok(pool)
}

pub fn build_app(state: AppState, cors_origin: &str) -> Result<Router, InvalidHeaderValue> {
    let cors = CorsLayer::new()
        .allow_origin(cors_origin.parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE]);

    Ok(Router::new()
        .route("/health", get(health))
        .nest("/api", todo::handler::routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}

async fn health() -> &'static str {
    "ok"
}
