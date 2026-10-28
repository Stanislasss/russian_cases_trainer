//! Account routes and the `CurrentUser` extractor.

use application::auth::{AuthService, RegisterInput, Session};
use axum::extract::{FromRequestParts, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::{Json, Router, routing::get, routing::post};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use domain::grammar::Locale;
use domain::user::User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::dto::UiLocale;
use crate::error::{AppError, Problem};
use crate::{Accounts, AppState};

pub const SESSION_COOKIE: &str = "session";

/// `/api/me`, plus registration and login when there are accounts.
pub fn routes(accounts: &Accounts) -> Router<AppState> {
    let router = Router::new().route("/api/me", get(me));
    match accounts {
        Accounts::Sessions { .. } => router
            .route("/api/auth/register", post(register))
            .route("/api/auth/login", post(login))
            .route("/api/auth/logout", post(logout)),
        Accounts::Local(_) => router,
    }
}

/// The account service and whether cookies are `Secure`. The account routes
/// only exist with sessions, so the local user never gets here.
fn sessions(state: &AppState) -> Result<(&AuthService, bool), AppError> {
    match &state.accounts {
        Accounts::Sessions {
            auth,
            cookie_secure,
        } => Ok((auth, *cookie_secure)),
        Accounts::Local(_) => Err(AppError::NotFound),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    email: String,
    password: String,
    display_name: String,
    /// Defaults to English.
    ui_locale: Option<UiLocale>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDto {
    id: Uuid,
    email: String,
    display_name: String,
    ui_locale: UiLocale,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        UserDto {
            id: user.id.0,
            email: user.email.as_str().to_owned(),
            display_name: user.display_name.as_str().to_owned(),
            ui_locale: user.ui_locale.into(),
        }
    }
}

/// Creates an account and logs it in (sets the session cookie).
#[utoipa::path(
    post, path = "/api/auth/register", tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, body = UserDto),
        (status = 409, description = "email already registered", body = Problem),
        (status = 422, description = "invalid input", body = Problem),
    )
)]
async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, CookieJar, Json<UserDto>), AppError> {
    let (auth, secure) = sessions(&state)?;
    let (user, session) = auth
        .register(RegisterInput {
            email: req.email,
            password: req.password,
            display_name: req.display_name,
            ui_locale: req.ui_locale.map_or(Locale::En, Locale::from),
        })
        .await?;
    let jar = jar.add(session_cookie(session, secure));
    Ok((StatusCode::CREATED, jar, Json(user.into())))
}

/// Logs in with email and password (sets the session cookie).
#[utoipa::path(
    post, path = "/api/auth/login", tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, body = UserDto),
        (status = 401, description = "wrong email or password", body = Problem),
    )
)]
async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<UserDto>), AppError> {
    let (auth, secure) = sessions(&state)?;
    let (user, session) = auth.login(&req.email, &req.password).await?;
    let jar = jar.add(session_cookie(session, secure));
    Ok((jar, Json(user.into())))
}

/// Ends the current session, if any, and clears the cookie.
#[utoipa::path(post, path = "/api/auth/logout", tag = "auth", responses((status = 204)))]
async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar), AppError> {
    let (auth, _) = sessions(&state)?;
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        auth.logout(cookie.value()).await?;
    }
    let jar = jar.remove(Cookie::build(SESSION_COOKIE).path("/"));
    Ok((StatusCode::NO_CONTENT, jar))
}

/// The logged-in user.
#[utoipa::path(
    get, path = "/api/me", tag = "auth",
    responses(
        (status = 200, body = UserDto),
        (status = 401, description = "not logged in", body = Problem),
    )
)]
async fn me(CurrentUser(user): CurrentUser) -> Json<UserDto> {
    Json(user.into())
}

fn session_cookie(session: Session, secure: bool) -> Cookie<'static> {
    let max_age = (session.expires_at - Utc::now()).num_seconds();
    Cookie::build((SESSION_COOKIE, session.token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(time::Duration::seconds(max_age))
        .build()
}

/// The user behind the request's session cookie, or the desktop app's local
/// user. Rejects with 401 otherwise, so taking it as a handler argument makes a
/// route require login.
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let auth = match &state.accounts {
            Accounts::Sessions { auth, .. } => auth,
            Accounts::Local(user) => return Ok(CurrentUser(user.clone())),
        };
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar
            .get(SESSION_COOKIE)
            .ok_or(AppError::Unauthenticated)?
            .value();
        auth.current_user(token)
            .await?
            .map(CurrentUser)
            .ok_or(AppError::Unauthenticated)
    }
}

/// Who is playing: like `CurrentUser`, but playing without logging in is fine.
pub struct Player(pub Option<User>);

impl FromRequestParts<AppState> for Player {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        match CurrentUser::from_request_parts(parts, state).await {
            Ok(CurrentUser(user)) => Ok(Player(Some(user))),
            Err(AppError::Unauthenticated) => Ok(Player(None)),
            Err(e) => Err(e),
        }
    }
}
