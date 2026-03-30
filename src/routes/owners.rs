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
        owner::{NewOwnerForm, OwnerEditItem, OwnerListItem},
    },
    state::AppState,
};

#[derive(Template)]
#[template(path = "owners.html")]
struct OwnersTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    owners: Vec<OwnerListItem>,
}

#[derive(Template)]
#[template(path = "owner_edit.html")]
struct OwnerEditTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    owner: OwnerEditItem,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> AppResult<Html<String>> {
    let owners = sqlx::query_as::<_, OwnerListItem>(
        r#"
        SELECT
            id,
            full_name,
            phone,
            COALESCE(email, '—') AS email,
            COALESCE(passport_no, '—') AS passport_no,
            COALESCE(notes, '—') AS notes,
            to_char(created_at, 'DD.MM.YYYY HH24:MI') AS registered_at
        FROM owners
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let template = OwnersTemplate {
        page_title: "Собственники".to_string(),
        nav_active: "owners".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        owners,
    };

    render_html(&template)
}

pub async fn edit_page(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(owner_id): Path<Uuid>,
) -> AppResult<Html<String>> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    let owner = sqlx::query_as::<_, OwnerEditItem>(
        r#"
        SELECT
            id,
            full_name,
            phone,
            COALESCE(email, '') AS email,
            COALESCE(passport_no, '') AS passport_no,
            COALESCE(notes, '') AS notes
        FROM owners
        WHERE id = $1
        "#,
    )
    .bind(owner_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Собственник не найден.".to_string()))?;

    let template = OwnerEditTemplate {
        page_title: "Редактирование собственника".to_string(),
        nav_active: "owners".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        owner,
    };

    render_html(&template)
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Form(form): Form<NewOwnerForm>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    validate_owner(&form)?;

    let owner_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO owners (full_name, phone, email, passport_no, notes)
        VALUES ($1, $2, NULLIF($3, ''), NULLIF($4, ''), NULLIF($5, ''))
        RETURNING id
        "#,
    )
    .bind(form.full_name.trim())
    .bind(form.phone.trim())
    .bind(form.email.as_deref().unwrap_or_default().trim())
    .bind(form.passport_no.as_deref().unwrap_or_default().trim())
    .bind(form.notes.as_deref().unwrap_or_default().trim())
    .fetch_one(&state.pool)
    .await?;

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "CREATE",
        "owners",
        Some(owner_id),
        &format!("Добавлен собственник '{}'", form.full_name.trim()),
    )
    .await?;

    Ok(Redirect::to("/app/owners").into_response())
}

pub async fn update(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(owner_id): Path<Uuid>,
    Form(form): Form<NewOwnerForm>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    validate_owner(&form)?;

    let result = sqlx::query(
        r#"
        UPDATE owners
        SET
            full_name = $2,
            phone = $3,
            email = NULLIF($4, ''),
            passport_no = NULLIF($5, ''),
            notes = NULLIF($6, '')
        WHERE id = $1
        "#,
    )
    .bind(owner_id)
    .bind(form.full_name.trim())
    .bind(form.phone.trim())
    .bind(form.email.as_deref().unwrap_or_default().trim())
    .bind(form.passport_no.as_deref().unwrap_or_default().trim())
    .bind(form.notes.as_deref().unwrap_or_default().trim())
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Собственник не найден.".to_string()));
    }

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "UPDATE",
        "owners",
        Some(owner_id),
        &format!("Обновлен собственник '{}'", form.full_name.trim()),
    )
    .await?;

    Ok(Redirect::to("/app/owners").into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(owner_id): Path<Uuid>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    let result = sqlx::query("DELETE FROM owners WHERE id = $1")
        .bind(owner_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Собственник не найден.".to_string()));
    }

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "DELETE",
        "owners",
        Some(owner_id),
        "Удален собственник",
    )
    .await?;

    Ok(Redirect::to("/app/owners").into_response())
}

fn validate_owner(form: &NewOwnerForm) -> AppResult<()> {
    if form.full_name.trim().is_empty() || form.phone.trim().is_empty() {
        return Err(AppError::BadRequest(
            "ФИО и телефон собственника обязательны.".to_string(),
        ));
    }

    Ok(())
}
