use askama::Template;
use axum::{
    extract::{Extension, Form, Path, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::{
    db::write_audit_event,
    error::{render_html, AppError, AppResult},
    models::{
        auth::CurrentUser,
        common::{LookupItem, UuidLookup},
        property::{NewPropertyForm, PropertyEditItem, PropertyFilter, PropertyListItem},
    },
    state::AppState,
};

#[derive(Template)]
#[template(path = "properties.html")]
struct PropertiesTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    filter_q: String,
    properties: Vec<PropertyListItem>,
    categories: Vec<LookupItem>,
    statuses: Vec<LookupItem>,
    conditions: Vec<LookupItem>,
    districts: Vec<LookupItem>,
    owners: Vec<UuidLookup>,
}

#[derive(Template)]
#[template(path = "property_edit.html")]
struct PropertyEditTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    property: PropertyEditItem,
    categories: Vec<LookupItem>,
    statuses: Vec<LookupItem>,
    conditions: Vec<LookupItem>,
    districts: Vec<LookupItem>,
    owners: Vec<UuidLookup>,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(filter): Query<PropertyFilter>,
) -> AppResult<Html<String>> {
    let properties = list_properties(&state, &filter).await?;
    let (categories, statuses, conditions, districts, owners) =
        load_property_lookups(&state).await?;

    let template = PropertiesTemplate {
        page_title: "Объекты недвижимости".to_string(),
        nav_active: "properties".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        filter_q: filter.q.unwrap_or_default(),
        properties,
        categories,
        statuses,
        conditions,
        districts,
        owners,
    };

    render_html(&template)
}

pub async fn edit_page(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(property_id): Path<Uuid>,
) -> AppResult<Html<String>> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    let property = sqlx::query_as::<_, PropertyEditItem>(
        r#"
        SELECT
            id,
            owner_id,
            category_id,
            status_id,
            condition_id,
            district_id,
            title,
            area::double precision AS area,
            price::double precision AS price,
            address,
            COALESCE(description, '') AS description
        FROM properties
        WHERE id = $1
        "#,
    )
    .bind(property_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Объект недвижимости не найден.".to_string()))?;

    let (categories, statuses, conditions, districts, owners) =
        load_property_lookups(&state).await?;

    let template = PropertyEditTemplate {
        page_title: "Редактирование объекта".to_string(),
        nav_active: "properties".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        property,
        categories,
        statuses,
        conditions,
        districts,
        owners,
    };

    render_html(&template)
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Form(form): Form<NewPropertyForm>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    validate_property(&form)?;

    let property_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO properties (
            owner_id,
            category_id,
            status_id,
            condition_id,
            district_id,
            title,
            area,
            price,
            address,
            description
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NULLIF($10, ''))
        RETURNING id
        "#,
    )
    .bind(form.owner_id)
    .bind(form.category_id)
    .bind(form.status_id)
    .bind(form.condition_id)
    .bind(form.district_id)
    .bind(form.title.trim())
    .bind(form.area)
    .bind(form.price)
    .bind(form.address.trim())
    .bind(form.description.as_deref().unwrap_or_default().trim())
    .fetch_one(&state.pool)
    .await?;

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "CREATE",
        "properties",
        Some(property_id),
        &format!("Добавлен объект недвижимости '{}'", form.title.trim()),
    )
    .await?;

    Ok(Redirect::to("/app/properties").into_response())
}

pub async fn update(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(property_id): Path<Uuid>,
    Form(form): Form<NewPropertyForm>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    validate_property(&form)?;

    let result = sqlx::query(
        r#"
        UPDATE properties
        SET
            owner_id = $2,
            category_id = $3,
            status_id = $4,
            condition_id = $5,
            district_id = $6,
            title = $7,
            area = $8,
            price = $9,
            address = $10,
            description = NULLIF($11, '')
        WHERE id = $1
        "#,
    )
    .bind(property_id)
    .bind(form.owner_id)
    .bind(form.category_id)
    .bind(form.status_id)
    .bind(form.condition_id)
    .bind(form.district_id)
    .bind(form.title.trim())
    .bind(form.area)
    .bind(form.price)
    .bind(form.address.trim())
    .bind(form.description.as_deref().unwrap_or_default().trim())
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Объект недвижимости не найден.".to_string(),
        ));
    }

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "UPDATE",
        "properties",
        Some(property_id),
        &format!("Обновлен объект недвижимости '{}'", form.title.trim()),
    )
    .await?;

    Ok(Redirect::to("/app/properties").into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(property_id): Path<Uuid>,
) -> AppResult<Response> {
    if !user.can_edit() {
        return Err(AppError::Forbidden);
    }

    let result = sqlx::query("DELETE FROM properties WHERE id = $1")
        .bind(property_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Объект недвижимости не найден.".to_string(),
        ));
    }

    write_audit_event(
        &state.pool,
        Some(user.user_id),
        "DELETE",
        "properties",
        Some(property_id),
        "Удален объект недвижимости",
    )
    .await?;

    Ok(Redirect::to("/app/properties").into_response())
}

async fn list_properties(
    state: &AppState,
    filter: &PropertyFilter,
) -> AppResult<Vec<PropertyListItem>> {
    let mut builder = QueryBuilder::<Postgres>::new(
        r#"
        SELECT
            p.id,
            p.title,
            pc.name AS category_name,
            ps.name AS status_name,
            o.full_name AS owner_name,
            d.name AS district_name,
            c.name AS condition_name,
            p.area::double precision AS area,
            p.price::double precision AS price,
            p.address,
            to_char(p.listed_at, 'DD.MM.YYYY') AS listed_at
        FROM properties p
        JOIN property_categories pc ON pc.id = p.category_id
        JOIN property_statuses ps ON ps.id = p.status_id
        JOIN property_conditions c ON c.id = p.condition_id
        JOIN districts d ON d.id = p.district_id
        JOIN owners o ON o.id = p.owner_id
        WHERE 1 = 1
        "#,
    );

    if let Some(query) = filter
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pattern = format!("%{}%", query.to_lowercase());
        builder
            .push(" AND (lower(p.title) LIKE ")
            .push_bind(pattern.clone())
            .push(" OR lower(p.address) LIKE ")
            .push_bind(pattern)
            .push(")");
    }

    if let Some(category_id) = filter.category_id {
        builder.push(" AND p.category_id = ").push_bind(category_id);
    }

    if let Some(status_id) = filter.status_id {
        builder.push(" AND p.status_id = ").push_bind(status_id);
    }

    if let Some(district_id) = filter.district_id {
        builder.push(" AND p.district_id = ").push_bind(district_id);
    }

    builder.push(" ORDER BY p.created_at DESC");

    Ok(builder
        .build_query_as::<PropertyListItem>()
        .fetch_all(&state.pool)
        .await?)
}

async fn load_property_lookups(
    state: &AppState,
) -> AppResult<(
    Vec<LookupItem>,
    Vec<LookupItem>,
    Vec<LookupItem>,
    Vec<LookupItem>,
    Vec<UuidLookup>,
)> {
    let categories =
        sqlx::query_as::<_, LookupItem>("SELECT id, name FROM property_categories ORDER BY name")
            .fetch_all(&state.pool)
            .await?;
    let statuses =
        sqlx::query_as::<_, LookupItem>("SELECT id, name FROM property_statuses ORDER BY id")
            .fetch_all(&state.pool)
            .await?;
    let conditions =
        sqlx::query_as::<_, LookupItem>("SELECT id, name FROM property_conditions ORDER BY name")
            .fetch_all(&state.pool)
            .await?;
    let districts = sqlx::query_as::<_, LookupItem>("SELECT id, name FROM districts ORDER BY name")
        .fetch_all(&state.pool)
        .await?;
    let owners = sqlx::query_as::<_, UuidLookup>(
        "SELECT id, full_name AS name FROM owners ORDER BY full_name",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok((categories, statuses, conditions, districts, owners))
}

fn validate_property(form: &NewPropertyForm) -> AppResult<()> {
    if form.title.trim().is_empty() || form.address.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Название и адрес объекта обязательны.".to_string(),
        ));
    }

    if form.area <= 0.0 || form.price < 0.0 {
        return Err(AppError::BadRequest(
            "Площадь должна быть больше нуля, стоимость не может быть отрицательной.".to_string(),
        ));
    }

    Ok(())
}
