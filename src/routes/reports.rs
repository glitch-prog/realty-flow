use askama::Template;
use axum::{
    body::Body,
    extract::{Extension, Form, Path, State},
    http::header,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    error::{render_html, AppError, AppResult},
    models::{
        auth::CurrentUser,
        report::{
            AuditLogRow, CatalogReportRow, ManagerPerformanceRow, MonthlySalesRow,
            OwnerPortfolioRow,
        },
    },
    state::AppState,
};

#[derive(Template)]
#[template(path = "reports.html")]
struct ReportsTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    catalog: Vec<CatalogReportRow>,
    owner_portfolio: Vec<OwnerPortfolioRow>,
    monthly_sales: Vec<MonthlySalesRow>,
    manager_performance: Vec<ManagerPerformanceRow>,
    sql_query: String,
    sql_headers: Vec<String>,
    sql_rows: Vec<Vec<String>>,
    sql_error: String,
    has_sql_result: bool,
    has_sql_error: bool,
}

#[derive(Template)]
#[template(path = "audit.html")]
struct AuditTemplate {
    page_title: String,
    nav_active: String,
    is_authenticated: bool,
    user_name: String,
    role_name: String,
    is_admin: bool,
    can_edit: bool,
    audit_rows: Vec<AuditLogRow>,
}

#[derive(Debug, Deserialize)]
pub struct AdminSqlForm {
    pub sql: String,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> AppResult<Html<String>> {
    let report_data = load_report_data(&state).await?;

    let template = ReportsTemplate {
        page_title: "Отчеты".to_string(),
        nav_active: "reports".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        catalog: report_data.catalog,
        owner_portfolio: report_data.owner_portfolio,
        monthly_sales: report_data.monthly_sales,
        manager_performance: report_data.manager_performance,
        sql_query: String::new(),
        sql_headers: Vec::new(),
        sql_rows: Vec::new(),
        sql_error: String::new(),
        has_sql_result: false,
        has_sql_error: false,
    };

    render_html(&template)
}

pub async fn execute_sql(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Form(form): Form<AdminSqlForm>,
) -> AppResult<Html<String>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden);
    }

    let report_data = load_report_data(&state).await?;
    let sql_query = form.sql.trim().to_string();

    let (sql_headers, sql_rows, sql_error) = match run_admin_query(&state, &sql_query).await {
        Ok((headers, rows)) => (headers, rows, String::new()),
        Err(error) => (Vec::new(), Vec::new(), format_error_message(error)),
    };
    let has_sql_result = !sql_headers.is_empty();
    let has_sql_error = !sql_error.is_empty();

    let template = ReportsTemplate {
        page_title: "Отчеты".to_string(),
        nav_active: "reports".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        catalog: report_data.catalog,
        owner_portfolio: report_data.owner_portfolio,
        monthly_sales: report_data.monthly_sales,
        manager_performance: report_data.manager_performance,
        sql_query,
        sql_headers,
        sql_rows,
        sql_error,
        has_sql_result,
        has_sql_error,
    };

    render_html(&template)
}

pub async fn export_report(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path((report_name, format)): Path<(String, String)>,
) -> AppResult<Response> {
    let export = build_predefined_export(&state, &user, &report_name).await?;
    Ok(render_export_response(
        &export.title,
        &export.headers,
        &export.rows,
        &format,
    ))
}

pub async fn export_sql(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(format): Path<String>,
    Form(form): Form<AdminSqlForm>,
) -> AppResult<Response> {
    if !user.is_admin() {
        return Err(AppError::Forbidden);
    }

    let sanitized = sanitize_select_sql(&form.sql)?;
    let (headers, rows) = run_admin_query(&state, &sanitized).await?;

    Ok(render_export_response(
        "custom-query",
        &headers,
        &rows,
        &format,
    ))
}

pub async fn audit_log(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> AppResult<Html<String>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden);
    }

    let audit_rows = load_audit_rows(&state).await?;

    let template = AuditTemplate {
        page_title: "Журнал действий".to_string(),
        nav_active: "audit".to_string(),
        is_authenticated: true,
        user_name: user.full_name.clone(),
        role_name: user.role_name.clone(),
        is_admin: user.is_admin(),
        can_edit: user.can_edit(),
        audit_rows,
    };

    render_html(&template)
}

struct ReportData {
    catalog: Vec<CatalogReportRow>,
    owner_portfolio: Vec<OwnerPortfolioRow>,
    monthly_sales: Vec<MonthlySalesRow>,
    manager_performance: Vec<ManagerPerformanceRow>,
}

struct ExportPayload {
    title: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

async fn load_report_data(state: &AppState) -> AppResult<ReportData> {
    let catalog = sqlx::query_as::<_, CatalogReportRow>(
        r#"
        SELECT
            property_title,
            category_name,
            status_name,
            district_name,
            owner_name,
            price::double precision AS price
        FROM vw_property_catalog
        ORDER BY price DESC
        LIMIT 10
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let owner_portfolio = sqlx::query_as::<_, OwnerPortfolioRow>(
        r#"
        SELECT
            owner_name,
            properties_count,
            active_deals,
            total_portfolio_value::double precision AS total_portfolio_value
        FROM vw_owner_portfolio
        ORDER BY total_portfolio_value DESC
        LIMIT 10
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let monthly_sales = sqlx::query_as::<_, MonthlySalesRow>(
        r#"
        SELECT
            period_label,
            completed_deals,
            total_amount::double precision AS total_amount,
            total_commission::double precision AS total_commission,
            payments_received::double precision AS payments_received
        FROM vw_sales_summary_by_month
        ORDER BY period_label DESC
        LIMIT 6
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let manager_performance = sqlx::query_as::<_, ManagerPerformanceRow>(
        r#"
        SELECT
            manager_name,
            completed_deals,
            total_amount::double precision AS total_amount,
            total_commission::double precision AS total_commission
        FROM vw_manager_performance
        ORDER BY total_amount DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(ReportData {
        catalog,
        owner_portfolio,
        monthly_sales,
        manager_performance,
    })
}

async fn load_audit_rows(state: &AppState) -> AppResult<Vec<AuditLogRow>> {
    Ok(sqlx::query_as::<_, AuditLogRow>(
        r#"
        SELECT
            COALESCE(u.full_name, 'Системное действие') AS actor_name,
            a.action_type,
            a.entity_name,
            a.details,
            to_char(a.created_at, 'DD.MM.YYYY HH24:MI:SS') AS created_at
        FROM audit_log a
        LEFT JOIN app_users u ON u.id = a.actor_user_id
        ORDER BY a.created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.pool)
    .await?)
}

async fn build_predefined_export(
    state: &AppState,
    user: &CurrentUser,
    report_name: &str,
) -> AppResult<ExportPayload> {
    match report_name {
        "catalog" => {
            let rows = load_report_data(state).await?.catalog;
            Ok(ExportPayload {
                title: "catalog".to_string(),
                headers: vec![
                    "Объект".to_string(),
                    "Категория".to_string(),
                    "Статус".to_string(),
                    "Район".to_string(),
                    "Собственник".to_string(),
                    "Цена".to_string(),
                ],
                rows: rows
                    .into_iter()
                    .map(|row| {
                        vec![
                            row.property_title,
                            row.category_name,
                            row.status_name,
                            row.district_name,
                            row.owner_name,
                            row.price.to_string(),
                        ]
                    })
                    .collect(),
            })
        }
        "owner-portfolio" => {
            let rows = load_report_data(state).await?.owner_portfolio;
            Ok(ExportPayload {
                title: "owner-portfolio".to_string(),
                headers: vec![
                    "Собственник".to_string(),
                    "Объекты".to_string(),
                    "Активные сделки".to_string(),
                    "Стоимость портфеля".to_string(),
                ],
                rows: rows
                    .into_iter()
                    .map(|row| {
                        vec![
                            row.owner_name,
                            row.properties_count.to_string(),
                            row.active_deals.to_string(),
                            row.total_portfolio_value.to_string(),
                        ]
                    })
                    .collect(),
            })
        }
        "monthly-sales" => {
            let rows = load_report_data(state).await?.monthly_sales;
            Ok(ExportPayload {
                title: "monthly-sales".to_string(),
                headers: vec![
                    "Период".to_string(),
                    "Сделки".to_string(),
                    "Сумма".to_string(),
                    "Комиссия".to_string(),
                    "Платежи".to_string(),
                ],
                rows: rows
                    .into_iter()
                    .map(|row| {
                        vec![
                            row.period_label,
                            row.completed_deals.to_string(),
                            row.total_amount.to_string(),
                            row.total_commission.to_string(),
                            row.payments_received.to_string(),
                        ]
                    })
                    .collect(),
            })
        }
        "manager-performance" => {
            let rows = load_report_data(state).await?.manager_performance;
            Ok(ExportPayload {
                title: "manager-performance".to_string(),
                headers: vec![
                    "Менеджер".to_string(),
                    "Завершенные сделки".to_string(),
                    "Объем".to_string(),
                    "Комиссия".to_string(),
                ],
                rows: rows
                    .into_iter()
                    .map(|row| {
                        vec![
                            row.manager_name,
                            row.completed_deals.to_string(),
                            row.total_amount.to_string(),
                            row.total_commission.to_string(),
                        ]
                    })
                    .collect(),
            })
        }
        "audit" => {
            if !user.is_admin() {
                return Err(AppError::Forbidden);
            }

            let rows = load_audit_rows(state).await?;
            Ok(ExportPayload {
                title: "audit".to_string(),
                headers: vec![
                    "Пользователь".to_string(),
                    "Действие".to_string(),
                    "Сущность".to_string(),
                    "Детали".to_string(),
                    "Дата".to_string(),
                ],
                rows: rows
                    .into_iter()
                    .map(|row| {
                        vec![
                            row.actor_name,
                            row.action_type,
                            row.entity_name,
                            row.details,
                            row.created_at,
                        ]
                    })
                    .collect(),
            })
        }
        _ => Err(AppError::NotFound(
            "Неизвестный отчет для экспорта.".to_string(),
        )),
    }
}

async fn run_admin_query(
    state: &AppState,
    sql: &str,
) -> AppResult<(Vec<String>, Vec<Vec<String>>)> {
    let sanitized = sanitize_select_sql(sql)?;
    let wrapped = format!(
        "SELECT row_to_json(result_row)::text AS row_json FROM ({}) AS result_row",
        sanitized
    );

    let json_rows = sqlx::query_scalar::<_, String>(&wrapped)
        .fetch_all(&state.pool)
        .await?;

    if json_rows.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut headers = Vec::new();
    let mut rows = Vec::new();

    for row_json in json_rows {
        let value: Value = serde_json::from_str(&row_json).map_err(|error| {
            AppError::Internal(format!("Не удалось разобрать результат SQL: {error}"))
        })?;
        let object = value.as_object().ok_or_else(|| {
            AppError::Internal("Ожидался объектный результат SQL-запроса.".to_string())
        })?;

        if headers.is_empty() {
            headers = object.keys().cloned().collect();
        }

        rows.push(
            headers
                .iter()
                .map(|key| json_cell_to_string(object.get(key).unwrap_or(&Value::Null)))
                .collect(),
        );
    }

    Ok((headers, rows))
}

fn sanitize_select_sql(sql: &str) -> AppResult<String> {
    let trimmed = sql.trim().trim_end_matches(';').trim();

    if trimmed.is_empty() {
        return Err(AppError::BadRequest("Введите SQL-запрос.".to_string()));
    }

    if trimmed.contains(';') {
        return Err(AppError::BadRequest(
            "Разрешен только один SELECT/WITH/VALUES-запрос без нескольких инструкций.".to_string(),
        ));
    }

    let lowered = trimmed.to_lowercase();
    if !(lowered.starts_with("select")
        || lowered.starts_with("with")
        || lowered.starts_with("values"))
    {
        return Err(AppError::BadRequest(
            "В режиме администратора через интерфейс разрешены только запросы SELECT, WITH и VALUES."
                .to_string(),
        ));
    }

    Ok(trimmed.to_string())
}

fn render_export_response(
    title: &str,
    headers: &[String],
    rows: &[Vec<String>],
    format: &str,
) -> Response {
    let (content_type, file_name, body) = match format {
        "csv" => (
            "text/csv; charset=utf-8",
            format!("{title}.csv"),
            format!("\u{feff}{}", render_csv(headers, rows)),
        ),
        "xls" => (
            "application/vnd.ms-excel; charset=utf-8",
            format!("{title}.xls"),
            render_excel_html(title, headers, rows),
        ),
        _ => {
            return AppError::BadRequest("Поддерживаются только форматы csv и xls.".to_string())
                .into_response()
        }
    };

    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{file_name}\""),
        )
        .body(Body::from(body))
        .unwrap_or_else(|error| {
            AppError::Internal(format!("Не удалось сформировать ответ экспорта: {error}"))
                .into_response()
        })
}

fn render_csv(headers: &[String], rows: &[Vec<String>]) -> String {
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(
        headers
            .iter()
            .map(|value| csv_escape(value))
            .collect::<Vec<_>>()
            .join(","),
    );
    for row in rows {
        lines.push(
            row.iter()
                .map(|value| csv_escape(value))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    lines.join("\r\n")
}

fn render_excel_html(title: &str, headers: &[String], rows: &[Vec<String>]) -> String {
    let header_html = headers
        .iter()
        .map(|item| format!("<th>{}</th>", html_escape(item)))
        .collect::<String>();
    let rows_html = rows
        .iter()
        .map(|row| {
            let cells = row
                .iter()
                .map(|cell| format!("<td>{}</td>", html_escape(cell)))
                .collect::<String>();
            format!("<tr>{cells}</tr>")
        })
        .collect::<String>();

    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>{}</title></head><body><table border=\"1\"><thead><tr>{}</tr></thead><tbody>{}</tbody></table></body></html>",
        html_escape(title),
        header_html,
        rows_html,
    )
}

fn csv_escape(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn json_cell_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

fn format_error_message(error: AppError) -> String {
    match error {
        AppError::BadRequest(message)
        | AppError::NotFound(message)
        | AppError::Internal(message) => message,
        AppError::Forbidden => "Недостаточно прав для выполнения SQL-запроса.".to_string(),
        AppError::Unauthorized => "Необходима авторизация.".to_string(),
        AppError::Db(error) => format!("Ошибка базы данных: {error}"),
        AppError::Template(error) => format!("Ошибка шаблона: {error}"),
    }
}
