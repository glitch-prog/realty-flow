pub mod auth;
pub mod clients;
pub mod dashboard;
pub mod deals;
pub mod owners;
pub mod properties;
pub mod reports;

use axum::{
    extract::Request,
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::CookieJar;

use crate::{models::auth::CurrentUser, state::AppState};

pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/", get(dashboard::index))
        .route(
            "/properties",
            get(properties::index).post(properties::create),
        )
        .route("/properties/:id/edit", get(properties::edit_page))
        .route("/properties/:id/update", post(properties::update))
        .route("/properties/:id/delete", post(properties::delete))
        .route("/owners", get(owners::index).post(owners::create))
        .route("/owners/:id/edit", get(owners::edit_page))
        .route("/owners/:id/update", post(owners::update))
        .route("/owners/:id/delete", post(owners::delete))
        .route("/clients", get(clients::index).post(clients::create))
        .route("/clients/:id/edit", get(clients::edit_page))
        .route("/clients/:id/update", post(clients::update))
        .route("/clients/:id/delete", post(clients::delete))
        .route("/deals", get(deals::index).post(deals::create))
        .route("/deals/:id/edit", get(deals::edit_page))
        .route("/deals/:id/update", post(deals::update))
        .route("/deals/:id/delete", post(deals::delete))
        .route("/reports", get(reports::index))
        .route("/reports/sql", post(reports::execute_sql))
        .route("/reports/sql/export/:format", post(reports::export_sql))
        .route(
            "/reports/export/:report_name/:format",
            get(reports::export_report),
        )
        .route("/audit", get(reports::audit_log))
        .route_layer(middleware::from_fn(auth_guard));

    Router::new()
        .route("/", get(|| async { Redirect::to("/app") }))
        .route("/health", get(auth::health))
        .route("/login", get(auth::login_page).post(auth::login))
        .route("/logout", post(auth::logout))
        .nest("/app", protected)
        .with_state(state)
}

async fn auth_guard(mut request: Request, next: Next) -> Response {
    let jar = CookieJar::from_headers(request.headers());

    let Some(user) = CurrentUser::from_jar(&jar) else {
        return Redirect::to("/login").into_response();
    };

    request.extensions_mut().insert(user);
    next.run(request).await
}
