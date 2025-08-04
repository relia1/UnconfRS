use crate::config::AppState;
use crate::middleware::auth::AuthSessionLayer;
use crate::models::session_tags_model::{add_session_tag, remove_session_tag, SessionTagError};
use crate::models::tags_model::Tag;
use crate::types::ApiStatusCode;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_macros::debug_handler;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddTagToSessionRequest {
    pub tag_id: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RemoveTagFromSessionRequest {
    pub tag_id: i32,
}

#[utoipa::path(
    post,
    path = "/api/v1/sessions/{session_id}/tags",
    request_body = AddTagToSessionRequest,
    responses(
        (status = 200, description = "Tag added to session", body = [Tag]),
        (status = 409, description = "Tag already applied to session", body = SessionTagError),
        (status = 403, description = "Unauthorized access", body = SessionTagError),
        (status = 404, description = "Tag or session not found", body = SessionTagError),
    )
)]
#[debug_handler]
/// Adds a tag to a session
///
/// This function is a handler for the route `POST /api/v1/sessions/{session_id}/tags`.
/// It adds a tag to a session in the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `auth_session` - Authentication session for authorization
/// - `session_id` - The id of the session to add the tag to
/// - `tag_id` - JSON body containing the tag ID to add
///
/// # Returns
/// `Response` with a status code of 200 OK and the updated list of tags for the session,
/// or an error response if the tag could not be added.
///
/// # Errors
/// If an error occurs while adding the tag (tag already applied, unauthorized access, etc.),
/// a session tag error response is returned.
pub async fn add_tag_for_session(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Path(session_id): Path<i32>,
    Json(request): Json<AddTagToSessionRequest>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let db_pool = &app_state_lock.unconf_data.read().await.unconf_db;

    match add_session_tag(db_pool, auth_session, session_id, request.tag_id).await {
        Ok(session_tags) => (StatusCode::OK, Json(session_tags)).into_response(),
        Err(e) => {
            let status = if e.to_string().contains("already applied") {
                StatusCode::CONFLICT
            } else if e.to_string().contains("does not have access") {
                StatusCode::FORBIDDEN
            } else if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            SessionTagError::response(ApiStatusCode::from(status), e)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/sessions/{session_id}/tags",
    request_body = RemoveTagFromSessionRequest,
    responses(
        (status = 200, description = "Tag removed from session", body = [Tag]),
        (status = 404, description = "Tag not found on session", body = SessionTagError),
        (status = 403, description = "Unauthorized access", body = SessionTagError),
    )
)]
#[debug_handler]
/// Removes a tag from a session
///
/// This function is a handler for the route `DELETE /api/v1/sessions/{session_id}/tags`.
/// It removes a tag from a session in the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `auth_session` - Authentication session for authorization
/// - `session_id` - The id of the session to remove the tag from
/// - `tag_id` - JSON body containing the tag ID to remove
///
/// # Returns
/// `Response` with a status code of 200 OK and the updated list of tags for the session,
/// or an error response if the tag could not be removed.
///
/// # Errors
/// If an error occurs while removing the tag (tag not found on session, unauthorized access, etc.),
/// a session tag error response is returned.
pub async fn remove_tag_for_session(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Path(session_id): Path<i32>,
    Json(request): Json<RemoveTagFromSessionRequest>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let db_pool = &app_state_lock.unconf_data.read().await.unconf_db;

    match remove_session_tag(db_pool, auth_session, session_id, request.tag_id).await {
        Ok(session_tags) => (StatusCode::OK, Json(session_tags)).into_response(),
        Err(e) => {
            let status = if e.to_string().contains("NonExistentTag") || e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("does not have access") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::BAD_REQUEST
            };
            SessionTagError::response(ApiStatusCode::from(status), e)
        }
    }
}