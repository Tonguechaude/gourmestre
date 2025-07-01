use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;

pub async fn create_pool() -> Pool {
    dotenvy::dotenv().ok();

    let mut cfg = Config::new();
    cfg.dbname = Some("Gourmestre".to_string());
    cfg.user = Some("u_gourmestre".to_string());
    cfg.password = Some("tongue".to_string());
    cfg.host = Some("localhost".to_string());

    cfg.create_pool(Some(Runtime::Tokio1), NoTls).expect("Failed to create DB pool")
}

pub async fn create_user(pool: &Pool, username: &str, password: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    client
        .execute(
            "INSERT INTO users (email, username, password_hash) VALUES ($1, $1, $2)",
            &[&username, &password],
        )
        .await?;
    Ok(())
}
