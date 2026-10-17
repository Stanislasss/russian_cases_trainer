//! HTTP layer: axum routes, request/response DTOs and the OpenAPI document.
//! Handlers only talk to the application use cases.

pub mod auth;
pub mod config;
pub mod dto;
pub mod error;
pub mod games;
pub mod local;
pub mod sentences;
pub mod speech;

use std::path::Path;
use std::sync::Arc;

use application::auth::AuthService;
use application::game::EndingsGame;
use application::sentences::SentenceGame;
use application::speech::{SpeechPack, SpeechService};
use axum::{Json, Router, routing::get};
use domain::user::User;
use serde::Serialize;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use utoipa::{OpenApi, ToSchema};

#[derive(Clone)]
pub struct AppState {
    pub accounts: Accounts,
    pub endings: EndingsGame,
    pub sentences: SentenceGame,
    pub speech: SpeechService,
    /// The desktop app's downloadable speech pack; `None` on the web, where
    /// Piper runs as a service.
    pub speech_pack: Option<Arc<dyn SpeechPack>>,
}

/// Who is playing.
#[derive(Clone)]
pub enum Accounts {
    /// Email + password accounts with session cookies (the web app).
    Sessions {
        auth: AuthService,
        /// Set the `Secure` flag on cookies; enable when served over HTTPS.
        cookie_secure: bool,
    },
    /// The desktop app's one local user: always there, no account routes.
    Local(User),
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Russian Cases Trainer API"),
    paths(
        health,
        config::config,
        auth::register,
        auth::login,
        auth::logout,
        auth::me,
        games::endings_batch,
        games::endings_attempts,
        sentences::sentences_batch,
        sentences::sentences_attempts,
        speech::speech,
        speech::install_pack,
        speech::remove_pack,
    ),
    components(schemas(Health, error::Problem, error::ErrorCode))
)]
pub struct ApiDoc;

/// The OpenAPI document, used by `server print-openapi` and the frontend codegen.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// The API, plus the built frontend from `static_dir` when given (the desktop
/// app serves both; on the web, nginx serves the frontend).
pub fn router(state: AppState, static_dir: Option<&Path>) -> Router {
    let mut router = Router::new()
        .route("/api/health", get(health))
        .route("/api/openapi.json", get(|| async { Json(openapi()) }))
        .merge(config::routes())
        .merge(auth::routes(&state.accounts))
        .merge(games::routes())
        .merge(sentences::routes())
        .merge(speech::routes(state.speech_pack.is_some()))
        .with_state(state);
    if let Some(dir) = static_dir {
        // Client-side routes (/rules/genitive) get the app itself.
        router = router
            .route("/api/{*rest}", axum::routing::any(error::not_found))
            .fallback_service(ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html"))));
    }
    router
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

#[cfg(test)]
mod tests;
