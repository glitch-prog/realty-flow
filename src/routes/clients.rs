use askama::Template;
use axum::{
    extract::{Extension, Form, Path, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use uuid::Uuid;

use crate::{
    db::write_audit_event,
    error::{render_html, AppError, AppResult},
    models::{
        auth::CurrentUser,
        client::{ClientEditItem, ClientListItem, NewClientForm},
        common::LookupItem,
    },
    state::AppState,
};

#[derive(Template)]
#[template(path = "clients.html")]
struct ClientsTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    client_types: Vec<LookupItem>,
    categories: Vec<LookupItem>,
    clients: Vec<ClientListItem>,
}

#[derive(Template)]
#[template(path = "client_edit.html")]
struct ClientEditTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    client_types: Vec<LookupItem>,
    categories: Vec<LookupItem>,
    client: ClientEditItem,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> AppResult<Html<String>> {
    let (client_types, categories) = load_client_lookups(&state).await?;
    let clients = sqlx::query_as::<_, ClientListItem>(
        r#"
        SELECT
            c.id,
            c.full_name,
            c.phone,
            COALESCE(c.email, '—') AS email,
            ct.name AS client_type_name,
            COALESCE(to_char(c.budget, 'FM999999999.00'), '—') AS budget_text,
            COALESCE(pc.name, 'Без предпочтения') AS preferred_category_name,
            COALESCE(c.notes, '—') AS notes,
            to_char(c.created_at, 'DD.MM.YYYY HH24:MI') AS created_at
        FROM clients c
        JOIN client_types ct ON ct.id = c.client_type_id
        LEFT JOIN property_categories pc ON pc.id = c.preferred_category_id
        ORDER BY c.created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let template = ClientsTemplate {
        page_title: "Клиенты".to_string(),
        nav_active: "clients".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        client_types,
        categories,
        clients,
    };

    render_html(&template)
}

pub async fn edit_page(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(client_id): Path<Uuid>,
) -> AppResult<Html<String>> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    let client = sqlx::query_as::<_, ClientEditItem>(
        r#"
        SELECT
            id,
            client_type_id,
            full_name,
            phone,
            COALESCE(email, '') AS email,
            COALESCE(budget::text, '') AS budget,
            COALESCE(preferred_category_id, 0) AS preferred_category_id,
            COALESCE(notes, '') AS notes
        FROM clients
        WHERE id = $1
        "#,
    )
    .bind(client_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Клиент не найден.".to_string()))?;

    let (client_types, categories) = load_client_lookups(&state).await?;

    let template = ClientEditTemplate {
        page_title: "Редактирование клиента".to_string(),
        nav_active: "clients".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        client_types,
        categories,
        client,
    };

    render_html(&template)
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Form(form): Form<NewClientForm>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    validate_client(&form)?;

    let client_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO clients (
            client_type_id,
            full_name,
            phone,
            email,
            budget,
            preferred_category_id,
            notes
        )
        VALUES ($1, $2, $3, NULLIF($4, ''), $5, $6, NULLIF($7, ''))
        RETURNING id
        "#,
    )
    .bind(form.client_type_id)
    .bind(form.full_name.trim())
    .bind(form.phone.trim())
    .bind(form.email.as_deref().unwrap_or_default().trim())
    .bind(form.budget)
    .bind(form.preferred_category_id)
    .bind(form.notes.as_deref().unwrap_or_default().trim())
    .fetch_one(&state.pool)
    .await?;

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "CREATE",
        "clients",
        Some(client_id),
        &format!("Добавлен клиент '{}'", form.full_name.trim()),
    )
    .await?;

    Ok(Redirect::to("/app/clients").into_response())
}

pub async fn update(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(client_id): Path<Uuid>,
    Form(form): Form<NewClientForm>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    validate_client(&form)?;

    let result = sqlx::query(
        r#"
        UPDATE clients
        SET
            client_type_id = $2,
            full_name = $3,
            phone = $4,
            email = NULLIF($5, ''),
            budget = $6,
            preferred_category_id = $7,
            notes = NULLIF($8, '')
        WHERE id = $1
        "#,
    )
    .bind(client_id)
    .bind(form.client_type_id)
    .bind(form.full_name.trim())
    .bind(form.phone.trim())
    .bind(form.email.as_deref().unwrap_or_default().trim())
    .bind(form.budget)
    .bind(form.preferred_category_id)
    .bind(form.notes.as_deref().unwrap_or_default().trim())
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Клиент не найден.".to_string()));
    }

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "UPDATE",
        "clients",
        Some(client_id),
        &format!("Обновлен клиент '{}'", form.full_name.trim()),
    )
    .await?;

    Ok(Redirect::to("/app/clients").into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(client_id): Path<Uuid>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    let result = sqlx::query("DELETE FROM clients WHERE id = $1")
        .bind(client_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Клиент не найден.".to_string()));
    }

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "DELETE",
        "clients",
        Some(client_id),
        "Удален клиент",
    )
    .await?;

    Ok(Redirect::to("/app/clients").into_response())
}

async fn load_client_lookups(state: &AppState) -> AppResult<(Vec<LookupItem>, Vec<LookupItem>)> {
    let client_types =
        sqlx::query_as::<_, LookupItem>("SELECT id, name FROM client_types ORDER BY id")
            .fetch_all(&state.pool)
            .await?;
    let categories =
        sqlx::query_as::<_, LookupItem>("SELECT id, name FROM property_categories ORDER BY name")
            .fetch_all(&state.pool)
            .await?;

    Ok((client_types, categories))
}

fn validate_client(form: &NewClientForm) -> AppResult<()> {
    if form.full_name.trim().is_empty() || form.phone.trim().is_empty() {
        return Err(AppError::BadRequest(
            "ФИО и телефон клиента обязательны.".to_string(),
        ));
    }

    if let Some(budget) = form.budget {
        if budget < 0.0 {
            return Err(AppError::BadRequest(
                "Бюджет клиента не может быть отрицательным.".to_string(),
            ));
        }
    }

    Ok(())
}
