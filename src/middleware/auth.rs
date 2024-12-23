use crate::config::AppState;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Middleware to check if the user is authorized
///
/// This function is a middleware that checks if the user is authorized to access the route. It
/// extracts the token from the cookie and compares it to the JWT secret in the app state.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `req` - The request to check for authorization
/// - `next` - The next middleware or handler to call
///
/// # Returns
/// If the user is authorized, the next middleware or handler is called otherwise an unauthorized
/// response is returned.
///
/// # Errors
/// If the user is not authorized, a response with a status code of 401 Unauthorized is returned.
pub async fn auth_middleware(
    State(app_state): State<Arc<RwLock<AppState>>>,
    req: Request,
    next: Next,
) -> Result<Response, Response> {
    let app_state_lock = app_state.read().await;
    let jwt_secret = app_state_lock.jwt_secret.read().await.clone();
    let headers = req.headers();
    match extract_cookie(headers, "token") {
        Some(token) if token == jwt_secret => Ok(next.run(req).await),
        _ => Err((StatusCode::UNAUTHORIZED, "Unauthorized").into_response()),
    }
}

/// Extracts a cookie from the request headers
///
/// This function extracts a cookie from the request headers by looking for the key in the cookie
/// header.
///
/// # Parameters
/// - `headers` - The headers from the request
/// - `key` - The key of the cookie to extract
///
/// # Returns
/// The value of the cookie if it exists, otherwise None
///
/// # Errors
/// If the cookie header is not found or the cookie key is not found, None is returned.
fn extract_cookie(headers: &HeaderMap, key: &str) -> Option<String> {
    headers.get("cookie").and_then(|cookie_header| {
        cookie_header.to_str().ok()
    })
        .and_then(|cookie_str| {
            cookie_str.split(';')
                .find_map(|cookie| {
                    let (cookie_key, cookie_value) = cookie.trim().split_once('=')?;
                    if cookie_key.trim() == key {
                        Some(
                            cookie_value.trim().to_string(),
                        )
                    } else {
                        None
                    }
                })
        })
}
