//! HTTP layer: axum routes, request/response DTOs and the OpenAPI document.
//! Handlers only talk to the application use cases.

pub mod auth;
pub mod dto;
pub mod error;
pub mod games;

use application::auth::AuthService;
use application::game::EndingsGame;
use axum::{Json, Router, routing::get};
use serde::Serialize;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use utoipa::{OpenApi, ToSchema};

#[derive(Clone)]
pub struct AppState {
    pub auth: AuthService,
    pub endings: EndingsGame,
    /// Set the `Secure` flag on cookies; enable when served over HTTPS.
    pub cookie_secure: bool,
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Russian Cases Trainer API"),
    paths(
        health,
        auth::register,
        auth::login,
        auth::logout,
        auth::me,
        games::endings_batch,
        games::endings_attempts,
    ),
    components(schemas(Health, error::Problem, error::ErrorCode))
)]
pub struct ApiDoc;

/// The OpenAPI document, used by `server print-openapi` and the frontend codegen.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/openapi.json", get(|| async { Json(openapi()) }))
        .merge(auth::routes())
        .merge(games::routes())
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}

#[derive(Serialize, ToSchema)]
pub struct Health {
    status: &'static str,
}

#[utoipa::path(get, path = "/api/health", responses((status = 200, body = Health)))]
async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}
