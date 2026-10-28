//! The player's preferences: the settings stored with their progress.

use application::preferences::Preferences as Stored;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AppState;
use crate::auth::CurrentUser;
use crate::error::{AppError, Problem};

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/api/preferences",
        get(get_preferences).put(put_preferences),
    )
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Preferences {
    /// A word answered right the last 3 times in a case comes up 4 times less
    /// often in that case. On by default.
    fewer_known_words: bool,
}

impl From<Stored> for Preferences {
    fn from(stored: Stored) -> Self {
        Preferences {
            fewer_known_words: stored.fewer_known_words,
        }
    }
}

impl From<Preferences> for Stored {
    fn from(dto: Preferences) -> Self {
        Stored {
            fewer_known_words: dto.fewer_known_words,
        }
    }
}

/// The logged-in player's preferences.
#[utoipa::path(
    get, path = "/api/preferences", tag = "preferences",
    responses(
        (status = 200, body = Preferences),
        (status = 401, description = "not logged in", body = Problem),
    )
)]
async fn get_preferences(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Preferences>, AppError> {
    let stored = state
        .preferences
        .get(user.id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(stored.into()))
}

/// Replaces the logged-in player's preferences.
#[utoipa::path(
    put, path = "/api/preferences", tag = "preferences",
    request_body = Preferences,
    responses(
        (status = 204),
        (status = 401, description = "not logged in", body = Problem),
    )
)]
async fn put_preferences(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<Preferences>,
) -> Result<StatusCode, AppError> {
    state
        .preferences
        .save(user.id, &req.into())
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(StatusCode::NO_CONTENT)
}
