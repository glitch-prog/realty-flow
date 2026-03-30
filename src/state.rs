use sqlx::PgPool;

use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub settings: Settings,
}

impl AppState {
    pub fn new(pool: PgPool, settings: Settings) -> Self {
        Self { pool, settings }
    }
}
