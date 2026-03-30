use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct NewOwnerForm {
    pub full_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub passport_no: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct OwnerListItem {
    pub id: Uuid,
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub passport_no: String,
    pub notes: String,
    pub registered_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct OwnerEditItem {
    pub id: Uuid,
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub passport_no: String,
    pub notes: String,
}
