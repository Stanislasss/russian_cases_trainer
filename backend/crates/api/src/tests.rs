//! The router as the desktop app runs it: local user, speech pack, static files.

use std::sync::Mutex;

use application::game::{Attempt, AttemptRepository, ExerciseRepository, GameFilter, GameItem};
use application::ports::{BoxError, RepoError};
use application::speech::{AudioCache, PackStatus, SpeechSynthesizer};
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use domain::grammar::Locale;
use domain::user::{DisplayName, Email, UserId};
use http_body_util::BodyExt;
use tower::ServiceExt;
use uuid::Uuid;

use super::*;

struct NoWords;

#[async_trait]
impl ExerciseRepository for NoWords {
    async fn zones(&self, _: &GameFilter, _: f64, _: usize) -> Result<Vec<String>, RepoError> {
        Ok(Vec::new())
    }
    async fn random_items(
        &self,
        _: &GameFilter,
        _: &[String],
        _: usize,
    ) -> Result<Vec<GameItem>, RepoError> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl AttemptRepository for NoWords {
    async fn record(&self, _: UserId, _: &str, _: &[Attempt]) -> Result<(), RepoError> {
        Ok(())
    }
}

#[async_trait]
impl SpeechSynthesizer for NoWords {
    fn voice(&self) -> &str {
        "none"
    }
    async fn synthesize(&self, _: &str) -> Result<Vec<u8>, BoxError> {
        Err("no speech".into())
    }
}

#[async_trait]
impl AudioCache for NoWords {
    async fn get(&self, _: &str) -> Option<Vec<u8>> {
        None
    }
    async fn put(&self, _: &str, _: &[u8]) -> Result<(), BoxError> {
        Ok(())
    }
}

/// A speech pack that only records what was asked of it.
struct FakePack(Mutex<PackStatus>);

#[async_trait]
impl SpeechPack for FakePack {
    fn status(&self) -> PackStatus {
        self.0.lock().unwrap().clone()
    }
    fn install(&self) {
        *self.0.lock().unwrap() = PackStatus::Installing { progress: 0.5 };
    }
    async fn remove(&self) -> Result<(), BoxError> {
        *self.0.lock().unwrap() = PackStatus::NotInstalled;
        Ok(())
    }
}

fn local_state() -> AppState {
    AppState {
        accounts: Accounts::Local(User {
            id: UserId(Uuid::nil()),
            email: Email::parse("me@localhost.local").unwrap(),
            display_name: DisplayName::parse("Me").unwrap(),
            ui_locale: Locale::En,
            created_at: Utc::now(),
        }),
        endings: EndingsGame::new(Arc::new(NoWords), Arc::new(NoWords)),
        sentences: SentenceGame::new(Arc::new(NoWords), Arc::new(NoWords)),
        speech: SpeechService::new(Arc::new(NoWords), Arc::new(NoWords)),
        speech_pack: Some(Arc::new(FakePack(Mutex::new(PackStatus::NotInstalled)))),
    }
}

async fn send(router: &Router, request: Request<Body>) -> (StatusCode, String) {
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&body).into_owned())
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

fn post(uri: &str) -> Request<Body> {
    Request::post(uri).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn the_local_user_is_always_there_and_has_no_account_routes() {
    let router = router(local_state(), None);

    let (status, body) = send(&router, get("/api/me")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#""display_name":"Me""#), "{body}");

    let (status, _) = send(&router, post("/api/auth/login")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn config_follows_the_speech_pack() {
    let router = router(local_state(), None);

    let (_, body) = send(&router, get("/api/config")).await;
    let config: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        config,
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "accounts": false,
            "speech": { "state": "not_installed" }
        })
    );

    let (status, _) = send(&router, post("/api/speech/pack")).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let (_, body) = send(&router, get("/api/config")).await;
    assert!(
        body.contains(r#"{"state":"installing","progress":0.5}"#),
        "{body}"
    );

    let delete = Request::delete("/api/speech/pack")
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&router, delete).await.0, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn without_a_pack_there_are_no_pack_routes() {
    let state = AppState {
        speech_pack: None,
        ..local_state()
    };
    let router = router(state, None);
    let (_, body) = send(&router, get("/api/config")).await;
    assert!(body.contains(r#""speech":{"state":"service"}"#), "{body}");
    assert_eq!(
        send(&router, post("/api/speech/pack")).await.0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn the_frontend_is_served_with_client_side_routes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "the app").unwrap();
    std::fs::create_dir(dir.path().join("assets")).unwrap();
    std::fs::write(dir.path().join("assets/main.js"), "code").unwrap();
    let router = router(local_state(), Some(dir.path()));

    assert_eq!(send(&router, get("/")).await.1, "the app");
    assert_eq!(send(&router, get("/rules/genitive")).await.1, "the app");
    assert_eq!(send(&router, get("/assets/main.js")).await.1, "code");
    let (status, body) = send(&router, get("/api/nothing")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains(r#""code":"not_found""#), "{body}");
}

#[tokio::test]
async fn only_the_apps_own_window_may_use_the_local_server() {
    let origin = "http://127.0.0.1:4000";
    let router = local::only_from(router(local_state(), None), origin);
    let request = |method: &str, host: &str, from: Option<&str>| {
        let mut request = Request::builder()
            .method(method)
            .uri("/api/speech/pack")
            .header(header::HOST, host);
        if let Some(from) = from {
            request = request.header(header::ORIGIN, from);
        }
        request.body(Body::empty()).unwrap()
    };
    let host = "127.0.0.1:4000";

    // DNS rebinding: a site whose name resolves to 127.0.0.1
    let (status, _) = send(&router, request("GET", "evil.example:4000", None)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // another site posting to it
    let (status, _) = send(&router, request("POST", host, Some("https://evil.example"))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // the app itself
    let (status, _) = send(&router, request("POST", host, Some(origin))).await;
    assert_eq!(status, StatusCode::ACCEPTED);
}

#[tokio::test]
async fn sentences_take_noun_levels_only() {
    let router = router(local_state(), None);
    let batch = |levels: &str| {
        Request::post("/api/games/sentences/batch")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(format!(r#"{{"levels":[{levels}]}}"#)))
            .unwrap()
    };

    let (status, body) = send(&router, batch(r#""basic_nouns""#)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, r#"{"zones":[],"items":[]}"#); // no words in this store

    let (status, body) = send(&router, batch(r#""basic_adjectives""#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains(r#""code":"invalid_game_request""#), "{body}");
}

#[tokio::test]
async fn an_updated_app_never_shows_a_cached_page() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "the app").unwrap();
    std::fs::create_dir(dir.path().join("assets")).unwrap();
    std::fs::write(dir.path().join("assets/index-abc123.js"), "code").unwrap();
    let router = router(local_state(), Some(dir.path()));
    let cache_control = |uri: &'static str| {
        let router = router.clone();
        async move {
            let response = router.oneshot(get(uri)).await.unwrap();
            response.headers()[header::CACHE_CONTROL]
                .to_str()
                .unwrap()
                .to_owned()
        }
    };

    assert_eq!(cache_control("/").await, "no-cache");
    assert_eq!(cache_control("/play/sentences").await, "no-cache");
    assert_eq!(
        cache_control("/assets/index-abc123.js").await,
        "public, max-age=31536000, immutable"
    );
}
