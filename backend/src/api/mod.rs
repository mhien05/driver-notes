mod auth;
mod backup;
mod docs;
mod entries;
mod shifts;

use crate::db::AppState;
use auth::{login, logout, me, register};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method, StatusCode, header},
    routing::{get, post},
};
use backup::{restore, snapshot};
use entries::{create_entry, delete_entry, list_entries, replace_entry};
use shifts::{delete_shift, end_shift, list_shifts, replace_shift, start_shift};
use tower_http::cors::CorsLayer;
use utoipa_swagger_ui::SwaggerUi;

pub(crate) use docs::openapi;

pub(crate) fn router(state: AppState, frontend_origin: Option<&str>) -> Router {
    let mut app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/auth/me", get(me))
        .route("/api/v1/snapshot", get(snapshot))
        .route(
            "/api/v1/restore",
            post(restore).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route("/api/v1/shifts", get(list_shifts).post(start_shift))
        .route(
            "/api/v1/shifts/{id}",
            axum::routing::put(replace_shift).delete(delete_shift),
        )
        .route("/api/v1/shifts/{id}/end", post(end_shift))
        .route("/api/v1/entries", get(list_entries).post(create_entry))
        .route(
            "/api/v1/entries/{id}",
            axum::routing::put(replace_entry).delete(delete_entry),
        )
        .with_state(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi()));
    if let Some(origin) = frontend_origin {
        let value =
            HeaderValue::from_str(origin).expect("FRONTEND_ORIGIN must be a valid header value");
        app = app.layer(
            CorsLayer::new()
                .allow_origin(value)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        );
    }
    app
}
