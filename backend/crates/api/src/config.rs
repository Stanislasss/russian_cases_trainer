//! What this installation offers, so the frontend can adapt (web or desktop).

use application::speech::PackStatus;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{Accounts, AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/config", get(config))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AppConfig {
    /// Accounts with login (web); false in the desktop app, which has one local user.
    accounts: bool,
    speech: SpeechStatus,
}

/// Whether words can be pronounced.
#[derive(Debug, Serialize, ToSchema, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SpeechStatus {
    /// A speech service is configured (web); requests may still fail if it's down.
    Service,
    /// Desktop: the speech pack can be downloaded.
    NotInstalled,
    /// Desktop: the speech pack is being downloaded; `progress` goes from 0 to 1.
    Installing { progress: f32 },
    /// Desktop: the speech pack is installed.
    Ready,
    /// Desktop: the last download failed.
    Failed { error: String },
}

impl From<PackStatus> for SpeechStatus {
    fn from(status: PackStatus) -> Self {
        match status {
            PackStatus::NotInstalled => SpeechStatus::NotInstalled,
            PackStatus::Installing { progress } => SpeechStatus::Installing { progress },
            PackStatus::Ready => SpeechStatus::Ready,
            PackStatus::Failed { error } => SpeechStatus::Failed { error },
        }
    }
}

/// What this installation offers.
#[utoipa::path(get, path = "/api/config", responses((status = 200, body = AppConfig)))]
pub async fn config(State(state): State<AppState>) -> Json<AppConfig> {
    Json(AppConfig {
        accounts: matches!(state.accounts, Accounts::Sessions { .. }),
        speech: state
            .speech_pack
            .as_ref()
            .map_or(SpeechStatus::Service, |pack| pack.status().into()),
    })
}
