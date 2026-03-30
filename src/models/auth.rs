use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: Uuid,
    pub username: String,
    pub full_name: String,
    pub role_code: String,
    pub role_name: String,
}

impl CurrentUser {
    pub fn from_jar(jar: &CookieJar) -> Option<Self> {
        let user_id = jar.get("user_id")?.value().parse().ok()?;

        Some(Self {
            user_id,
            username: jar.get("username")?.value().to_string(),
            full_name: jar.get("full_name")?.value().to_string(),
            role_code: jar.get("role_code")?.value().to_string(),
            role_name: jar.get("role_name")?.value().to_string(),
        })
    }

    pub fn is_admin(&self) -> bool {
        self.role_code == "admin"
    }

    pub fn can_edit(&self) -> bool {
        matches!(self.role_code.as_str(), "admin" | "manager" | "realtor")
    }
}

#[derive(Debug, FromRow)]
pub struct AuthUserRecord {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub password_plain: String,
    pub role_code: String,
    pub role_name: String,
}
