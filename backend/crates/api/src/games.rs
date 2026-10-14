//! Game routes.

use application::game::{Attempt, Batch, GameItem};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::post};
use domain::game::LevelId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AppState;
use crate::auth::CurrentUser;
use crate::dto::{GrammaticalCase, GrammaticalNumber, Level, UiLocale};
use crate::error::{AppError, Problem};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/games/endings/batch", post(endings_batch))
        .route("/api/games/endings/attempts", post(endings_attempts))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EndingsBatchRequest {
    /// Words are drawn from any of these levels.
    levels: Vec<Level>,
    case: GrammaticalCase,
    /// 1–50, default 10.
    count: Option<usize>,
    /// Language of the translations (falls back to English).
    locale: UiLocale,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EndingsBatch {
    /// Ending zones for the board, in paradigm order; "" is the zero ending.
    /// Identical for every batch with the same levels and case.
    zones: Vec<String>,
    items: Vec<EndingsItem>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EndingsItem {
    lexeme_id: i64,
    /// Dictionary form with stress mark (U+0301).
    stressed_lemma: String,
    translation: Option<String>,
    number: GrammaticalNumber,
    /// Ending of the target form; always one of the zones.
    answer: String,
    /// All endings that count as correct (alternative forms such as книгою).
    accepted: Vec<String>,
    /// The full target form with stress mark, for feedback.
    stressed_form: String,
}

impl From<GameItem> for EndingsItem {
    fn from(item: GameItem) -> Self {
        EndingsItem {
            lexeme_id: item.lexeme_id,
            stressed_lemma: item.stressed_lemma,
            translation: item.translation,
            number: item.number.into(),
            answer: item.answer,
            accepted: item.accepted,
            stressed_form: item.stressed_form,
        }
    }
}

impl From<Batch> for EndingsBatch {
    fn from(batch: Batch) -> Self {
        EndingsBatch {
            zones: batch.zones,
            items: batch.items.into_iter().map(Into::into).collect(),
        }
    }
}

/// A batch of random words for the endings game, with their answers.
#[utoipa::path(
    post, path = "/api/games/endings/batch", tag = "games",
    request_body = EndingsBatchRequest,
    responses(
        (status = 200, body = EndingsBatch),
        (status = 422, description = "no level selected or bad count", body = Problem),
    )
)]
async fn endings_batch(
    State(state): State<AppState>,
    Json(req): Json<EndingsBatchRequest>,
) -> Result<Json<EndingsBatch>, AppError> {
    let levels: Vec<LevelId> = req.levels.into_iter().map(Into::into).collect();
    let batch = state
        .endings
        .new_batch(
            &levels,
            req.case.into(),
            req.count.unwrap_or(10),
            req.locale.into(),
        )
        .await?;
    Ok(Json(batch.into()))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EndingsAttempts {
    /// 1–100 answers.
    attempts: Vec<EndingsAttempt>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EndingsAttempt {
    lexeme_id: i64,
    case: GrammaticalCase,
    number: GrammaticalNumber,
    /// The ending the player chose.
    answer: String,
    correct: bool,
}

/// Records the logged-in player's answers.
#[utoipa::path(
    post, path = "/api/games/endings/attempts", tag = "games",
    request_body = EndingsAttempts,
    responses(
        (status = 204),
        (status = 401, description = "not logged in", body = Problem),
        (status = 422, description = "too many/few attempts or unknown word", body = Problem),
    )
)]
async fn endings_attempts(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<EndingsAttempts>,
) -> Result<StatusCode, AppError> {
    let attempts: Vec<Attempt> = req
        .attempts
        .into_iter()
        .map(|a| Attempt {
            lexeme_id: a.lexeme_id,
            case: a.case.into(),
            number: a.number.into(),
            answer: a.answer,
            correct: a.correct,
        })
        .collect();
    state.endings.record_attempts(user.id, &attempts).await?;
    Ok(StatusCode::NO_CONTENT)
}
