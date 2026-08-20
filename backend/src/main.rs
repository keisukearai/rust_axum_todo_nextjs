use todo_backend::state::AppState;
use todo_backend::{build_app, config::Config, connect_database};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("todo_backend=debug,tower_http=info,sqlx=warn")
        }))
        .init();

    let config = Config::from_env()?;
    let pool = connect_database(&config).await?;
    let app = build_app(AppState::new(pool), &config.cors_origin)?;

    let addr = config.bind_address();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
