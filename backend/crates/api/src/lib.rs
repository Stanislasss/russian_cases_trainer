//! HTTP layer: axum routes, request/response DTOs and the OpenAPI document.

use axum::{Json, Router, routing::get};
use serde::Serialize;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    info(title = "Russian Cases Trainer API"),
    paths(health),
    components(schemas(Health))
)]
pub struct ApiDoc;

/// The OpenAPI document, used by `server print-openapi` and the frontend codegen.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/openapi.json", get(|| async { Json(openapi()) }))
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
