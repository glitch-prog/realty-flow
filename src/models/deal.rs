use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct NewDealForm {
    pub property_id: Uuid,
    pub client_id: Uuid,
    pub deal_type_id: i32,
    pub status_id: i32,
    pub amount: f64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DealListItem {
    pub id: Uuid,
    pub property_title: String,
    pub client_name: String,
    pub manager_name: String,
    pub deal_type_name: String,
    pub deal_status_name: String,
    pub amount: f64,
    pub commission: f64,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct DealEditItem {
    pub id: Uuid,
    pub property_id: Uuid,
    pub client_id: Uuid,
    pub deal_type_id: i32,
    pub status_id: i32,
    pub amount: f64,
    pub commission: f64,
    pub start_date: String,
    pub end_date: String,
    pub notes: String,
}
