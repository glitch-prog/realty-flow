use serde::{de::Error as DeError, Deserialize, Deserializer};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct PropertyFilter {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub q: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub category_id: Option<i32>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub status_id: Option<i32>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub district_id: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewPropertyForm {
    pub owner_id: Uuid,
    pub category_id: i32,
    pub status_id: i32,
    pub condition_id: i32,
    pub district_id: i32,
    pub title: String,
    pub area: f64,
    pub price: f64,
    pub address: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PropertyListItem {
    pub id: Uuid,
    pub title: String,
    pub category_name: String,
    pub status_name: String,
    pub owner_name: String,
    pub district_name: String,
    pub condition_name: String,
    pub area: f64,
    pub price: f64,
    pub address: String,
    pub listed_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct PropertyEditItem {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub category_id: i32,
    pub status_id: i32,
    pub condition_id: i32,
    pub district_id: i32,
    pub title: String,
    pub area: f64,
    pub price: f64,
    pub address: String,
    pub description: String,
}

fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = Option::<String>::deserialize(deserializer)?;

    match value.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some(value) => value.parse::<T>().map(Some).map_err(D::Error::custom),
    }
}
