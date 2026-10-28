//! HTTP layer: axum routes, request/response DTOs and the OpenAPI document.
//! Handlers only talk to the application use cases.

pub mod auth;
pub mod config;
pub mod dto;
pub mod error;
pub mod games;
pub mod local;
pub mod preferences;
pub mod sentences;
pub mod speech;

use std::path::Path;
use std::sync::Arc;

use application::auth::AuthService;
use application::game::EndingsGame;
use application::preferences::PreferenceRepository;
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
    /// The settings each player keeps with their progress.
    pub preferences: Arc<dyn PreferenceRepository>,
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
        preferences::get_preferences,
        preferences::put_preferences,
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
        .merge(preferences::routes())
        .merge(speech::routes(state.speech_pack.is_some()))
        .with_state(state);
    if let Some(dir) = static_dir {
        // Client-side routes (/rules/genitive) get the app itself.
        router = router
            .route("/api/{*rest}", axum::routing::any(error::not_found))
            .fallback_service(ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html"))))
            .layer(axum::middleware::map_response(static_caching));
    }
    router
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}

/// The app's files: the page itself must be checked again every time, or an
/// updated app keeps showing the previous version from the webview's cache;
/// the bundles in /assets carry a content hash in their name and never change.
async fn static_caching(
    uri: axum::http::Uri,
    mut response: axum::response::Response,
) -> axum::response::Response {
    use axum::http::{HeaderValue, header::CACHE_CONTROL};
    let path = uri.path();
    if !path.starts_with("/api/") && !response.headers().contains_key(CACHE_CONTROL) {
        let policy = if path.starts_with("/assets/") {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache"
        };
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(policy));
    }
    response
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
