//! Pronunciation route.

use axum::extract::{Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::{Router, routing::get};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::AppState;
use crate::error::{AppError, Problem};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/speech", get(speech))
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
