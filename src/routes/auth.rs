use askama::Template;
use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde_json::{json, Value};

use crate::{
    db::write_audit_event,
    error::{render_html, AppResult},
    models::auth::{AuthUserRecord, CurrentUser, LoginForm},
    state::AppState,
};

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    page_title: String,
    nav_active: String,
    app_name: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    error_message: String,
}

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "realtyflow-coursework" }))
}

pub async fn login_page(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    if CurrentUser::from_jar(&jar).is_some() {
        return Ok(Redirect::to("/app").into_response());
    }

    Ok(render_login(&state, "")?.into_response())
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> AppResult<Response> {
    let user = sqlx::query_as::<_, AuthUserRecord>(
        r#"
        SELECT
            u.id,
            u.username,
            u.full_name,
            u.password_plain,
            r.code AS role_code,
            r.name AS role_name
        FROM app_users u
        JOIN user_roles r ON r.id = u.role_id
        WHERE u.username = $1
          AND u.is_active = TRUE
          AND (
              u.password_plain = $2
              OR u.password_plain = encode(digest($2, 'sha256'), 'hex')
          )
        "#,
    )
    .bind(form.username.trim())
    .bind(form.password.trim())
    .fetch_optional(&state.pool)
    .await?;

    let Some(user) = user else {
        return Ok(render_login(&state, "Неверный логин или пароль.")?.into_response());
    };

    write_audit_event(
        &state.pool,
        Some(user.id),
        "LOGIN",
        "app_users",
        Some(user.id),
        "Пользователь выполнил вход в систему",
    )
    .await?;

    let response = (
        jar.add(build_cookie("user_id", user.id.to_string()))
            .add(build_cookie("username", user.username.clone()))
            .add(build_cookie("full_name", user.full_name.clone()))
            .add(build_cookie("role_code", user.role_code.clone()))
            .add(build_cookie("role_name", user.role_name.clone())),
        Redirect::to("/app"),
    )
        .into_response();

    Ok(response)
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    if let Some(user) = CurrentUser::from_jar(&jar) {
        write_audit_event(
            &state.pool,
            Some(user.user_id),
            "LOGOUT",
            "app_users",
            Some(user.user_id),
            "Пользователь завершил сеанс",
        )
        .await?;
    }

    let response = (
        jar.add(build_cookie("user_id", String::new()))
            .add(build_cookie("username", String::new()))
            .add(build_cookie("full_name", String::new()))
            .add(build_cookie("role_code", String::new()))
            .add(build_cookie("role_name", String::new())),
        Redirect::to("/login"),
    )
        .into_response();

    Ok(response)
}

fn render_login(state: &AppState, error_message: &str) -> AppResult<axum::response::Html<String>> {
    let template = LoginTemplate {
        page_title: "Вход в систему".to_string(),
        nav_active: String::new(),
        app_name: state.settings.app_name.clone(),
        is_authenticated: false,
        user_name: String::new(),
        role_name: String::new(),
        is_admin: false,
        can_edit: false,
        error_message: error_message.to_string(),
    };

    render_html(&template)
}

fn build_cookie(name: &str, value: String) -> Cookie<'static> {
    let mut cookie = Cookie::new(name.to_string(), value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie
}
