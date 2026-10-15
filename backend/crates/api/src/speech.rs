//! Pronunciation route, and the desktop app's speech pack.

use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::{Router, routing::get, routing::post};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::AppState;
use crate::error::{AppError, Problem};

/// `/api/speech`, plus installing and removing the pack where there is one.
pub fn routes(speech_pack: bool) -> Router<AppState> {
    let router = Router::new().route("/api/speech", get(speech));
    if speech_pack {
        router.route("/api/speech/pack", post(install_pack).delete(remove_pack))
    } else {
        router
    }
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct SpeechQuery {
    /// A Russian word or short form, ideally with its stress mark (кни́гой).
    text: String,
}

/// The pronunciation of a Russian word, as WAV audio.
#[utoipa::path(
    get, path = "/api/speech", tag = "speech",
    params(SpeechQuery),
    responses(
        (status = 200, description = "WAV audio", content_type = "audio/wav", body = Vec<u8>),
        (status = 422, description = "not a short Russian text", body = Problem),
        (status = 503, description = "speech synthesis is unavailable", body = Problem),
    )
)]
async fn speech(
    State(state): State<AppState>,
    Query(query): Query<SpeechQuery>,
) -> Result<impl IntoResponse, AppError> {
    let audio = state.speech.say(&query.text).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "audio/wav"),
            // The same text always sounds the same (for a given voice).
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        audio,
    ))
}

/// Desktop app: starts downloading the speech pack. Follow its progress in
/// `/api/config`.
#[utoipa::path(
    post, path = "/api/speech/pack", tag = "speech",
    responses((status = 202), (status = 404, description = "not the desktop app", body = Problem))
)]
pub async fn install_pack(State(state): State<AppState>) -> Result<StatusCode, AppError> {
    state.speech_pack.ok_or(AppError::NotFound)?.install();
    Ok(StatusCode::ACCEPTED)
}

/// Desktop app: deletes the speech pack.
#[utoipa::path(
    delete, path = "/api/speech/pack", tag = "speech",
    responses((status = 204), (status = 404, description = "not the desktop app", body = Problem))
)]
pub async fn remove_pack(State(state): State<AppState>) -> Result<StatusCode, AppError> {
    let pack = state.speech_pack.ok_or(AppError::NotFound)?;
    pack.remove().await.map_err(AppError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}
