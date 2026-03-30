use askama::Template;
use axum::{
    extract::{Extension, State},
    response::Html,
};

use crate::{
    error::{render_html, AppResult},
    models::{
        auth::CurrentUser,
        report::{DashboardStats, RecentDealRow},
    },
    state::AppState,
};

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    stats: DashboardStats,
    recent_deals: Vec<RecentDealRow>,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> AppResult<Html<String>> {
    let total_properties = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM properties")
        .fetch_one(&state.pool)
        .await?;

    let free_properties = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM properties p
        JOIN property_statuses ps ON ps.id = p.status_id
        WHERE ps.code = 'free'
        "#,
    )
    .fetch_one(&state.pool)
    .await?;

    let total_clients = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM clients")
        .fetch_one(&state.pool)
        .await?;

    let active_deals = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM deals d
        JOIN deal_statuses ds ON ds.id = d.status_id
        WHERE ds.code IN ('registered', 'in_progress')
        "#,
    )
    .fetch_one(&state.pool)
    .await?;

    let monthly_revenue = sqlx::query_scalar::<_, f64>(
        r#"
        SELECT COALESCE(SUM(p.amount), 0)::double precision
        FROM payments p
        WHERE date_trunc('month', p.paid_at) = date_trunc('month', CURRENT_DATE)
        "#,
    )
    .fetch_one(&state.pool)
    .await?;

    let recent_deals = sqlx::query_as::<_, RecentDealRow>(
        r#"
        SELECT
            d.id,
            p.title AS property_title,
            c.full_name AS client_name,
            u.full_name AS manager_name,
            dt.name AS deal_type_name,
            ds.name AS deal_status_name,
            d.amount::double precision AS amount,
            to_char(d.start_date, 'DD.MM.YYYY') AS start_date
        FROM deals d
        JOIN properties p ON p.id = d.property_id
        JOIN clients c ON c.id = d.client_id
        JOIN app_users u ON u.id = d.manager_id
        JOIN deal_types dt ON dt.id = d.deal_type_id
        JOIN deal_statuses ds ON ds.id = d.status_id
        ORDER BY d.created_at DESC
        LIMIT 8
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let template = DashboardTemplate {
        page_title: "Панель управления".to_string(),
        nav_active: "dashboard".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        stats: DashboardStats {
            total_properties,
            free_properties,
            total_clients,
            active_deals,
            monthly_revenue,
        },
        recent_deals,
    };

    render_html(&template)
}
