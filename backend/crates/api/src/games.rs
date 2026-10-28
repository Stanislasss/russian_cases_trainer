//! Game routes.

use application::game::{Attempt, Batch, GameItem};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::post};
use domain::game::LevelId;
use domain::grammar::Locale;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AppState;
use crate::auth::{CurrentUser, Player};
use crate::dto::{
    AnswerType, DeclensionParadigm, GrammaticalCase, GrammaticalGender, GrammaticalNumber, Level,
    NounAnimacy,
};
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
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EndingsBatch {
    /// Whether zones and answers are endings or whole forms.
    answer_kind: AnswerType,
    /// Zones for the board: endings in paradigm order ("" is the zero ending),
    /// or whole forms. Identical for every batch with the same levels and case.
    zones: Vec<String>,
    items: Vec<EndingsItem>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EndingsItem {
    lexeme_id: i64,
    /// Dictionary form with stress mark (U+0301).
    stressed_lemma: String,
    /// Meanings per language; Russian ones are definitions. Show the UI
    /// language's, falling back to English.
    translations: Translations,
    number: GrammaticalNumber,
    /// For adjectives, pronouns and surnames: the gender of the form (singular).
    gender: Option<GrammaticalGender>,
    /// For adjective-like words in the accusative: animacy of the noun they go with.
    animacy: Option<NounAnimacy>,
    /// Ending (or whole form) of the target form; always one of the zones.
    answer: String,
    /// Everything that counts as correct (alternative forms such as книгою, него).
    accepted: Vec<String>,
    /// The full target form with stress mark, for feedback.
    stressed_form: String,
    /// The table the word follows, for explaining mistakes; absent for irregular words.
    paradigm: Option<DeclensionParadigm>,
}

/// A word's meanings per UI language; missing where the dictionaries have none.
#[derive(Debug, Default, Serialize, ToSchema)]
pub struct Translations {
    en: Option<String>,
    fr: Option<String>,
    de: Option<String>,
    /// A definition in Russian rather than a translation.
    ru: Option<String>,
}

impl From<Vec<(Locale, String)>> for Translations {
    fn from(texts: Vec<(Locale, String)>) -> Self {
        let mut translations = Translations::default();
        for (locale, text) in texts {
            let slot = match locale {
                Locale::En => &mut translations.en,
                Locale::Fr => &mut translations.fr,
                Locale::De => &mut translations.de,
                Locale::Ru => &mut translations.ru,
            };
            *slot = Some(text);
        }
        translations
    }
}

impl From<GameItem> for EndingsItem {
    fn from(item: GameItem) -> Self {
        EndingsItem {
            lexeme_id: item.lexeme_id,
            stressed_lemma: item.stressed_lemma,
            translations: Translations::from(item.translations),
            number: item.number.into(),
            gender: item.gender.map(Into::into),
            animacy: item.animacy.map(Into::into),
            answer: item.answer,
            accepted: item.accepted,
            stressed_form: item.stressed_form,
            paradigm: item.paradigm.map(Into::into),
        }
    }
}

impl From<Batch> for EndingsBatch {
    fn from(batch: Batch) -> Self {
        EndingsBatch {
            answer_kind: batch.answer.into(),
            zones: batch.zones,
            items: batch.items.into_iter().map(Into::into).collect(),
        }
    }
}

/// A batch of random words for the endings game, with their answers. Words
/// the player keeps getting right come up less often (see `/api/preferences`).
#[utoipa::path(
    post, path = "/api/games/endings/batch", tag = "games",
    request_body = EndingsBatchRequest,
    responses(
        (status = 200, body = EndingsBatch),
        (status = 422, description = "no level, levels that can't be mixed, or bad count", body = Problem),
    )
)]
async fn endings_batch(
    State(state): State<AppState>,
    Player(player): Player,
    Json(req): Json<EndingsBatchRequest>,
) -> Result<Json<EndingsBatch>, AppError> {
    let levels: Vec<LevelId> = req.levels.into_iter().map(Into::into).collect();
    let batch = state
        .endings
        .new_batch(
            &levels,
            req.case.into(),
            req.count.unwrap_or(10),
            player.map(|p| p.id),
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
    gender: Option<GrammaticalGender>,
    animacy: Option<NounAnimacy>,
    /// The ending (or whole form) the player chose.
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
    let attempts: Vec<Attempt> = req.into();
    state.endings.record_attempts(user.id, &attempts).await?;
    Ok(StatusCode::NO_CONTENT)
}

impl From<EndingsAttempts> for Vec<Attempt> {
    fn from(req: EndingsAttempts) -> Self {
        req.attempts
            .into_iter()
            .map(|a| Attempt {
                lexeme_id: a.lexeme_id,
                case: a.case.into(),
                number: a.number.into(),
                gender: a.gender.map(Into::into),
                animacy: a.animacy.map(Into::into),
                answer: a.answer,
                correct: a.correct,
            })
            .collect()
    }
}
