//! The desktop app's server listens on 127.0.0.1, where web pages open in the
//! user's browser could reach it too. Only the app's own window may use it.

use std::sync::Arc;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};

/// Rejects requests not addressed to `origin` (`http://127.0.0.1:<port>`): a
/// different `Host` (DNS rebinding) or, for changes, a different `Origin`
/// (another site posting to it).
pub fn only_from(router: Router, origin: &str) -> Router {
    let origin: Arc<str> = origin.trim_end_matches('/').into();
    router.layer(middleware::from_fn_with_state(origin, guard))
}

async fn guard(State(origin): State<Arc<str>>, request: Request, next: Next) -> Response {
    if allowed(&request, &origin) {
        next.run(request).await
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

fn allowed(request: &Request, origin: &str) -> bool {
    let value = |name| request.headers().get(name).and_then(|v| v.to_str().ok());
    let host = origin.split_once("://").map_or(origin, |(_, host)| host);
    let reads = matches!(*request.method(), Method::GET | Method::HEAD);
    value(header::HOST) == Some(host)
        && (reads || value(header::ORIGIN).is_none_or(|o| o == origin))
}
