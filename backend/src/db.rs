use std::time::Duration;

use sqlx::{PgPool, postgres::PgPoolOptions};

const ATTEMPTS: u32 = 15;

pub async fn connect(url: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    let mut attempt = 0;
    let pool = loop {
        match PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(url)
            .await
        {
            Ok(pool) => break pool,
            Err(error) if attempt + 1 >= ATTEMPTS => return Err(error.into()),
            Err(_) => {
                attempt += 1;
                tracing::warn!(attempt, "database is not ready");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    };
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn ping(pool: &PgPool) -> bool {
    sqlx::query("select 1").execute(pool).await.is_ok()
}
