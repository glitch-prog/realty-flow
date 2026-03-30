use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    Db(sqlx::Error),
    Template(askama::Error),
    BadRequest(String),
    Unauthorized,
    Forbidden,
    NotFound(String),
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::Db(value)
    }
}

impl From<askama::Error> for AppError {
    fn from(value: askama::Error) -> Self {
        Self::Template(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Db(error) => map_db_error(error),
            Self::Template(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка шаблона: {error}"),
            ),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Необходима авторизация".to_string(),
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "Недостаточно прав для выполнения операции".to_string(),
            ),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };

        (
            status,
            Html(format!("<h1>{}</h1><p>{}</p>", status.as_u16(), message)),
        )
            .into_response()
    }
}

pub fn render_html<T: askama::Template>(template: &T) -> AppResult<Html<String>> {
    Ok(Html(template.render()?))
}

fn map_db_error(error: sqlx::Error) -> (StatusCode, String) {
    if let sqlx::Error::Database(db_error) = &error {
        match db_error.code().as_deref() {
            Some("23503") => {
                return (
                    StatusCode::BAD_REQUEST,
                    "Операция нарушает ссылочную целостность: запись используется в связанных данных."
                        .to_string(),
                );
            }
            Some("23505") => {
                return (
                    StatusCode::BAD_REQUEST,
                    "Операция нарушает уникальность данных: такая запись уже существует."
                        .to_string(),
                );
            }
            Some("23514") => {
                return (
                    StatusCode::BAD_REQUEST,
                    "Значения не прошли проверку ограничений базы данных.".to_string(),
                );
            }
            _ => {}
        }
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Ошибка базы данных: {error}"),
    )
}
