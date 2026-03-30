use std::{str::FromStr, time::Duration};

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, PgConnection, PgPool,
};
use uuid::Uuid;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::from_str(database_url)?
        .application_name("realtyflow-coursework")
        .options([("lc_messages", "C"), ("client_encoding", "UTF8")]);

    // Validate connectivity first so startup surfaces the real PostgreSQL error
    // instead of timing out while waiting for the pool to open a connection.
    let connection = PgConnection::connect_with(&options).await?;
    drop(connection);

    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .max_connections(10)
        .connect_with(options)
        .await
}

pub async fn write_audit_event(
    pool: &PgPool,
    actor_user_id: Option<Uuid>,
    action_type: &str,
    entity_name: &str,
    entity_id: Option<Uuid>,
    details: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_user_id, action_type, entity_name, entity_id, details)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(actor_user_id)
    .bind(action_type)
    .bind(entity_name)
    .bind(entity_id)
    .bind(details)
    .execute(pool)
    .await?;

    Ok(())
}
