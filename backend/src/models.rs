use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<i16>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        s.parse::<i16>().map(Some).map_err(serde::de::Error::custom)
    }
}

fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_str() {
        "true" | "on" => Ok(true),
        _ => Ok(false),
    }
}

#[derive(Deserialize)]
pub struct ApiRequest {
    pub data: NewRestaurant,
}

#[derive(Serialize)]
pub struct Restaurant {
    pub id: i32,
    pub owner_id: i32,
    pub name: String,
    pub city: String,
    pub rating: Option<i16>,
    pub description: Option<String>,
    pub is_favorite: bool,
}

impl From<Row> for Restaurant {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            owner_id: row.get("owner_id"),
            name: row.get("name"),
            city: row.get("city"),
            rating: row.get("rating"),
            description: row.get("description"),
            is_favorite: row.get("is_favorite"),
        }
    }
}

#[derive(Deserialize)]
pub struct NewRestaurant {
    pub name: String,
    pub city: String,
    pub description: Option<String>,

    #[serde(deserialize_with = "empty_string_as_none", default)]
    pub rating: Option<i16>,
    
    #[serde(deserialize_with = "bool_from_string", default)]
    pub is_favorite: bool,
}

