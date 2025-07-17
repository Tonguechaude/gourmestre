use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use bcrypt::{hash, DEFAULT_COST};
use bcrypt::verify;
use uuid::Uuid;

use crate::models::{NewRestaurant, Restaurant};

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

    let hashed = hash(password, DEFAULT_COST)?;

    client
        .execute(
            "INSERT INTO users (email, username, password_hash) VALUES ($1, $1, $2)",
            &[&username, &hashed],
        )
        .await?;

    Ok(())
}

pub async fn verify_user(pool: &Pool, username: &str, password: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;

    let row = client
        .query_opt("SELECT password_hash FROM users WHERE username = $1 OR email = $1", &[&username])
        .await?;

    if let Some(row) = row {
        let hash: &str = row.get("password_hash");
        Ok(verify(password, hash)?)
    } else {
        Ok(false)
    }
}

pub async fn create_session(pool: &Pool, username: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;

    let row = client
        .query_one("SELECT id FROM users WHERE username = $1 OR email = $1", &[&username])
        .await?;
    let user_id: i32 = row.get("id");

    let token = Uuid::new_v4().to_string();

    client
        .execute(
            "INSERT INTO sessions (token, user_id) VALUES ($1, $2)",
            &[&token, &user_id],
        )
        .await?;

    Ok(token)
}

pub async fn get_user_from_session(pool: &Pool, token: &str) -> Result<Option<i32>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    let row = client
        .query_opt("SELECT user_id FROM sessions WHERE token = $1", &[&token])
        .await?;

    Ok(row.map(|r| r.get("user_id")))
}

pub async fn get_restaurants_for_user(pool: &Pool, user_id: i32, favorites_only: bool, limit: Option<i64>) -> Result<Vec<Restaurant>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let mut query = "SELECT * FROM restaurants WHERE owner_id = $1".to_string();
    if favorites_only {
        query.push_str(" AND is_favorite = TRUE");
    }
    query.push_str(" ORDER BY created_at DESC");

    // Si une limite est spécifiée, on l'ajoute à la requête
    if let Some(l) = limit {
        query.push_str(&format!(" LIMIT {}", l));
    }

    let rows = client
        .query(&query, &[&user_id])
        .await?;

    let restaurants = rows.into_iter().map(Restaurant::from).collect();

    Ok(restaurants)
}

pub async fn create_restaurant(pool: &Pool, user_id: i32, new_restaurant: &NewRestaurant) -> Result<Restaurant, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    let row = client
        .query_one(
            "INSERT INTO restaurants (owner_id, name, city, rating, description, is_favorite) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
            &[&user_id, &new_restaurant.name, &new_restaurant.city, &new_restaurant.rating, &new_restaurant.description, &new_restaurant.is_favorite],
        )
        .await?;

    Ok(Restaurant::from(row))
}

