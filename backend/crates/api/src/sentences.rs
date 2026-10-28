//! Sentence game routes.

use application::game::Attempt;
use application::sentences::{SentenceBatch as Batch, SentenceItem as Item};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::post};
use domain::game::LevelId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AppState;
use crate::auth::{CurrentUser, Player};
use crate::dto::{GrammaticalCase, Level};
use crate::error::{AppError, Problem};
use crate::games::{EndingsAttempts, EndingsItem, Translations};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/games/sentences/batch", post(sentences_batch))
        .route("/api/games/sentences/attempts", post(sentences_attempts))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SentencesBatchRequest {
    /// Noun levels the words come from.
    levels: Vec<Level>,
    /// 1–50, default 10. Fewer may come back when no word fits a frame.
    count: Option<usize>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SentencesBatch {
    /// The ending buttons of each case asked.
    zones: Vec<CaseZones>,
    items: Vec<SentenceItem>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CaseZones {
    case: GrammaticalCase,
    /// In paradigm order; "" is the zero ending.
    endings: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SentenceItem {
    template_id: String,
    /// Stress-marked Russian before and after the noun.
    before: String,
    after: String,
    /// The case the noun takes here.
    case: GrammaticalCase,
    /// Why: one of the case's uses, as in the rules (e.g. "about", "absence").
    reason: String,
    /// The sentence in en/fr/de, with ___ for the noun; no Russian.
    translations: Translations,
    /// The noun, its answer and everything needed to explain a wrong ending.
    word: EndingsItem,
}

impl From<Item> for SentenceItem {
    fn from(item: Item) -> Self {
        let t = item.template;
        SentenceItem {
            template_id: t.id.to_owned(),
            before: item.before,
            after: t.after.to_owned(),
            case: t.case.into(),
            reason: t.reason.to_owned(),
            translations: t
                .translations
                .iter()
                .map(|(locale, text)| (*locale, (*text).to_owned()))
                .collect::<Vec<_>>()
                .into(),
            word: item.word.into(),
        }
    }
}

impl From<Batch> for SentencesBatch {
    fn from(batch: Batch) -> Self {
        SentencesBatch {
            zones: batch
                .zones
                .into_iter()
                .map(|(case, endings)| CaseZones {
                    case: case.into(),
                    endings,
                })
                .collect(),
            items: batch.items.into_iter().map(Into::into).collect(),
        }
    }
}

/// A batch of sentences, each with one noun to put in the right case and form.
/// Words the player keeps getting right come up less often (see `/api/preferences`).
#[utoipa::path(
    post, path = "/api/games/sentences/batch", tag = "games",
    request_body = SentencesBatchRequest,
    responses(
        (status = 200, body = SentencesBatch),
        (status = 422, description = "no level, a level that isn't nouns, or bad count", body = Problem),
    )
)]
pub async fn sentences_batch(
    State(state): State<AppState>,
    Player(player): Player,
    Json(req): Json<SentencesBatchRequest>,
) -> Result<Json<SentencesBatch>, AppError> {
    let levels: Vec<LevelId> = req.levels.into_iter().map(Into::into).collect();
    let batch = state
        .sentences
        .new_batch(&levels, req.count.unwrap_or(10), player.map(|p| p.id))
        .await?;
    Ok(Json(batch.into()))
}

/// Records the player's sentence answers; `correct` means the case and the
/// ending were both right the first time.
#[utoipa::path(
    post, path = "/api/games/sentences/attempts", tag = "games",
    request_body = EndingsAttempts,
    responses(
        (status = 204),
        (status = 401, description = "not logged in", body = Problem),
        (status = 422, description = "too many/few attempts or unknown word", body = Problem),
    )
)]
pub async fn sentences_attempts(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<EndingsAttempts>,
) -> Result<StatusCode, AppError> {
    let attempts: Vec<Attempt> = req.into();
    state.sentences.record_attempts(user.id, &attempts).await?;
    Ok(StatusCode::NO_CONTENT)
}
