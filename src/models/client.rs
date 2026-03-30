use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct NewClientForm {
    pub client_type_id: i32,
    pub full_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub budget: Option<f64>,
    pub preferred_category_id: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ClientListItem {
    pub id: Uuid,
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub client_type_name: String,
    pub budget_text: String,
    pub preferred_category_name: String,
    pub notes: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ClientEditItem {
    pub id: Uuid,
    pub client_type_id: i32,
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub budget: String,
    pub preferred_category_id: i32,
    pub notes: String,
}
