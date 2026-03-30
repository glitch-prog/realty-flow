mod config;
mod db;
mod error;
mod models;
mod routes;
mod state;

use std::{io, net::SocketAddr};

use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{db::create_pool, routes::router, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "realtyflow_coursework=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = config::Settings::from_env();
    let pool = create_pool(&settings.database_url).await.map_err(|error| {
        io::Error::other(format!(
            "Не удалось подключиться к PostgreSQL. Проверьте DATABASE_URL, запущен ли сервер и доступен ли порт 5432: {error}"
        ))
    })?;
    let state = AppState::new(pool, settings.clone());

    let app = router(state)
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = settings.address().parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Server started on http://{}", settings.address());
    axum::serve(listener, app).await?;

    Ok(())
}
