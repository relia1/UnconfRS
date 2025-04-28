use crate::config::AppState;
use crate::middleware::auth::AuthSessionLayer;
use crate::models::session_voting_model::{decrement_vote, increment_vote, SessionVoteError};
use crate::types::ApiStatusCode;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_macros::debug_handler;
use std::sync::Arc;
use tokio::sync::RwLock;

#[utoipa::path(
    put,
    path = "/api/v1/sessions/{id}/increment",
    responses(
        (status = 200, description = "Updated session", body = ()),
        (status = 409, description = "Conflict", body = SessionVoteError),
    )
)]
#[debug_handler]
/// Increments the vote count for a session
///
/// This function is a handler for the route `PUT /api/v1/sessions/{id}/increment`. It increments the
/// vote count for a session in the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `session_id` - The id of the session to increment the vote count for
///
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the session was updated or an error
/// response if the session vote could not be updated.
///
/// # Errors
/// If an error occurs while updating the session vote, a session error response with a status code of
/// 400 Bad Request is returned.
pub async fn add_vote_for_session(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Path(session_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match increment_vote(write_lock, auth_session, session_id).await {
        Ok(sessions_user_voted_for) => (StatusCode::OK, Json(sessions_user_voted_for)).into_response(),
        Err(e) => SessionVoteError::response(ApiStatusCode::from(StatusCode::CONFLICT), e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/sessions/{id}/increment",
    responses(
        (status = 200, description = "Updated session", body = ()),
        (status = 409, description = "Conflict", body = SessionVoteError),
    )
)]
#[debug_handler]
/// Decrements the vote count for a session
///
/// This function is a handler for the route `PUT /api/v1/sessions/{id}/decrement`. It decrements the
/// vote count for a session in the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `session_id` - The id of the session to decrement the vote count for
///
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the session was updated or an error
/// response if the session vote could not be updated.
///
/// # Errors
/// If an error occurs while updating the session vote, a session error response with a status code of
/// 400 Bad Request is returned.
pub async fn subtract_vote_for_session(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Path(session_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match decrement_vote(write_lock, auth_session, session_id).await {
        Ok(sessions_user_voted_for) => (StatusCode::OK, Json(sessions_user_voted_for)).into_response(),
        Err(e) => SessionVoteError::response(ApiStatusCode::from(StatusCode::CONFLICT), e),
    }
}