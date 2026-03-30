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
        common::{LookupItem, UuidLookup},
        deal::{DealEditItem, DealListItem, NewDealForm},
    },
    state::AppState,
};

#[derive(Template)]
#[template(path = "deals.html")]
struct DealsTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    properties: Vec<UuidLookup>,
    clients: Vec<UuidLookup>,
    deal_types: Vec<LookupItem>,
    deal_statuses: Vec<LookupItem>,
    deals: Vec<DealListItem>,
}

#[derive(Template)]
#[template(path = "deal_edit.html")]
struct DealEditTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    properties: Vec<UuidLookup>,
    clients: Vec<UuidLookup>,
    deal_types: Vec<LookupItem>,
    deal_statuses: Vec<LookupItem>,
    deal: DealEditItem,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> AppResult<Html<String>> {
    let (properties, clients, deal_types, deal_statuses) = load_deal_lookups(&state).await?;
    let deals = list_deals(&state).await?;

    let template = DealsTemplate {
        page_title: "Сделки".to_string(),
        nav_active: "deals".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        properties,
        clients,
        deal_types,
        deal_statuses,
        deals,
    };

    render_html(&template)
}

pub async fn edit_page(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(deal_id): Path<Uuid>,
) -> AppResult<Html<String>> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    let deal = sqlx::query_as::<_, DealEditItem>(
        r#"
        SELECT
            id,
            property_id,
            client_id,
            deal_type_id,
            status_id,
            amount::double precision AS amount,
            commission::double precision AS commission,
            to_char(start_date, 'YYYY-MM-DD') AS start_date,
            COALESCE(to_char(end_date, 'YYYY-MM-DD'), '') AS end_date,
            COALESCE(notes, '') AS notes
        FROM deals
        WHERE id = $1
        "#,
    )
    .bind(deal_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Сделка не найдена.".to_string()))?;

    let (properties, clients, deal_types, deal_statuses) = load_deal_lookups(&state).await?;

    let template = DealEditTemplate {
        page_title: "Редактирование сделки".to_string(),
        nav_active: "deals".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        properties,
        clients,
        deal_types,
        deal_statuses,
        deal,
    };

    render_html(&template)
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Form(form): Form<NewDealForm>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    validate_deal(&form)?;

    let deal_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO deals (
            property_id,
            client_id,
            manager_id,
            deal_type_id,
            status_id,
            amount,
            start_date,
            end_date,
            notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7::date, NULLIF($8, '')::date, NULLIF($9, ''))
        RETURNING id
        "#,
    )
    .bind(form.property_id)
    .bind(form.client_id)
    .bind(user.user_id)
    .bind(form.deal_type_id)
    .bind(form.status_id)
    .bind(form.amount)
    .bind(form.start_date.trim())
    .bind(form.end_date.clone().unwrap_or_default())
    .bind(form.notes.as_deref().unwrap_or_default().trim())
    .fetch_one(&state.pool)
    .await?;

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "CREATE",
        "deals",
        Some(deal_id),
        "Зарегистрирована новая сделка",
    )
    .await?;

    Ok(Redirect::to("/app/deals").into_response())
}

pub async fn update(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(deal_id): Path<Uuid>,
    Form(form): Form<NewDealForm>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    validate_deal(&form)?;

    let result = sqlx::query(
        r#"
        UPDATE deals
        SET
            property_id = $2,
            client_id = $3,
            deal_type_id = $4,
            status_id = $5,
            amount = $6,
            start_date = $7::date,
            end_date = NULLIF($8, '')::date,
            notes = NULLIF($9, '')
        WHERE id = $1
        "#,
    )
    .bind(deal_id)
    .bind(form.property_id)
    .bind(form.client_id)
    .bind(form.deal_type_id)
    .bind(form.status_id)
    .bind(form.amount)
    .bind(form.start_date.trim())
    .bind(form.end_date.clone().unwrap_or_default())
    .bind(form.notes.as_deref().unwrap_or_default().trim())
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Сделка не найдена.".to_string()));
    }

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "UPDATE",
        "deals",
        Some(deal_id),
        "Обновлена сделка",
    )
    .await?;

    Ok(Redirect::to("/app/deals").into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(deal_id): Path<Uuid>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    let result = sqlx::query("DELETE FROM deals WHERE id = $1")
        .bind(deal_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Сделка не найдена.".to_string()));
    }

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "DELETE",
        "deals",
        Some(deal_id),
        "Удалена сделка",
    )
    .await?;

    Ok(Redirect::to("/app/deals").into_response())
}

async fn load_deal_lookups(
    state: &AppState,
) -> AppResult<(
    Vec<UuidLookup>,
    Vec<UuidLookup>,
    Vec<LookupItem>,
    Vec<LookupItem>,
)> {
    let properties = sqlx::query_as::<_, UuidLookup>(
        r#"
        SELECT
            p.id,
            (p.title || ' | ' || p.address) AS name
        FROM properties p
        JOIN property_statuses ps ON ps.id = p.status_id
        WHERE ps.code <> 'archived'
        ORDER BY p.created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await?;
    let clients = sqlx::query_as::<_, UuidLookup>(
        "SELECT id, full_name AS name FROM clients ORDER BY full_name",
    )
    .fetch_all(&state.pool)
    .await?;
    let deal_types = sqlx::query_as::<_, LookupItem>("SELECT id, name FROM deal_types ORDER BY id")
        .fetch_all(&state.pool)
        .await?;
    let deal_statuses =
        sqlx::query_as::<_, LookupItem>("SELECT id, name FROM deal_statuses ORDER BY id")
            .fetch_all(&state.pool)
            .await?;

    Ok((properties, clients, deal_types, deal_statuses))
}

async fn list_deals(state: &AppState) -> AppResult<Vec<DealListItem>> {
    Ok(sqlx::query_as::<_, DealListItem>(
        r#"
        SELECT
            d.id,
            p.title AS property_title,
            c.full_name AS client_name,
            u.full_name AS manager_name,
            dt.name AS deal_type_name,
            ds.name AS deal_status_name,
            d.amount::double precision AS amount,
            d.commission::double precision AS commission,
            to_char(d.start_date, 'DD.MM.YYYY') AS start_date,
            COALESCE(to_char(d.end_date, 'DD.MM.YYYY'), '—') AS end_date
        FROM deals d
        JOIN properties p ON p.id = d.property_id
        JOIN clients c ON c.id = d.client_id
        JOIN app_users u ON u.id = d.manager_id
        JOIN deal_types dt ON dt.id = d.deal_type_id
        JOIN deal_statuses ds ON ds.id = d.status_id
        ORDER BY d.created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await?)
}

fn validate_deal(form: &NewDealForm) -> AppResult<()> {
    if form.amount < 0.0 {
        return Err(AppError::BadRequest(
            "Сумма сделки не может быть отрицательной.".to_string(),
        ));
    }

    if form.start_date.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Дата начала сделки обязательна.".to_string(),
        ));
    }

    Ok(())
}
