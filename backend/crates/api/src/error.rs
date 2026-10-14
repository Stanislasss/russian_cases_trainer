//! Errors as RFC 7807 problem documents. `code` is stable and meant for the
//! frontend to pick a translated message; `title` is a fallback in English.

use application::auth::AuthError;
use application::game::GameError;
use application::speech::SpeechError;
use axum::Json;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use domain::user::ValidationError;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Game(#[from] GameError),
    #[error(transparent)]
    Speech(#[from] SpeechError),
    #[error("not logged in")]
    Unauthenticated,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Problem {
    /// Always `about:blank`; use `code` to tell problems apart.
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub title: String,
    pub status: u16,
    pub code: ErrorCode,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidEmail,
    InvalidPassword,
    InvalidDisplayName,
    EmailTaken,
    InvalidCredentials,
    Unauthenticated,
    InvalidGameRequest,
    InvalidSpeechText,
    SpeechUnavailable,
    Internal,
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, ErrorCode) {
        use ErrorCode::*;
        match self {
            AppError::Auth(AuthError::Validation(e)) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                match e {
                    ValidationError::InvalidEmail => InvalidEmail,
                    ValidationError::PasswordLength => InvalidPassword,
                    ValidationError::DisplayNameLength => InvalidDisplayName,
                },
            ),
            AppError::Auth(AuthError::EmailTaken) => (StatusCode::CONFLICT, EmailTaken),
            AppError::Auth(AuthError::InvalidCredentials) => {
                (StatusCode::UNAUTHORIZED, InvalidCredentials)
            }
            AppError::Unauthenticated => (StatusCode::UNAUTHORIZED, Unauthenticated),
            AppError::Game(GameError::Internal(_)) => (StatusCode::INTERNAL_SERVER_ERROR, Internal),
            AppError::Game(_) => (StatusCode::UNPROCESSABLE_ENTITY, InvalidGameRequest),
            AppError::Speech(SpeechError::InvalidText) => {
                (StatusCode::UNPROCESSABLE_ENTITY, InvalidSpeechText)
            }
            AppError::Speech(SpeechError::Unavailable(_)) => {
                (StatusCode::SERVICE_UNAVAILABLE, SpeechUnavailable)
            }
            AppError::Auth(AuthError::Internal(_)) => (StatusCode::INTERNAL_SERVER_ERROR, Internal),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        let title = if status.is_server_error() {
            tracing::error!(error = %self, "request failed");
            "internal server error".to_owned()
        } else {
            self.to_string()
        };
        let problem = Problem {
            kind: "about:blank",
            title,
            status: status.as_u16(),
            code,
        };
        (
            status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            Json(problem),
        )
            .into_response()
    }
}
