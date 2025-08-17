use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppState;
use crate::middleware::auth::{AuthInfo, AuthSessionLayer};
use crate::models::sessions_model::{add, add_for_user, delete, get, get_all_sessions, update, Session, SessionAddedForUser, SessionErr, SessionError};
use crate::types::ApiStatusCode;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::{debug_handler, Extension};
use serde_json::json;

#[utoipa::path(
    get,
    path = "/api/v1/sessions",
    params(
        ("page" = i32, Query, description = "Page", minimum = 1),
        ("limit" = i32, Query, description = "Limit", minimum = 1)
    ),
    responses(
        (status = 200, description = "List sessions", body = Vec<Session>),
        (status = 404, description = "No sessions in that range")
    )
)]
#[debug_handler]
/// Retrieves a list of sessions
///
/// This function is a handler for the route `GET /api/v1/sessions`. It retrieves a list of sessions
/// from the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with a status code of 200 OK and a JSON body containing the list of sessions or an
/// error response if no sessions are found.
///
/// # Errors
/// If an error occurs while retrieving the sessions, a session error response with a status code
/// of 404 Not Found is returned.
pub async fn sessions(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match get_all_sessions(read_lock).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => SessionError::response(
            ApiStatusCode::from(StatusCode::NOT_FOUND),
            Box::new(SessionErr::DoesNotExist(e.to_string())),
        ),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/sessions/{id}",
    responses(
        (status = 200, description = "Return specified session", body = Session),
        (status = 404, description = "No session with this id", body = SessionError),
    )
)]
#[debug_handler]
/// Retrieves a session by id
///
/// This function is a handler for the route `GET /api/v1/sessions/{id}`. It retrieves a session
/// from the database by its id.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `session_id` - The id of the session to retrieve
///
/// # Returns
/// `Response` with a status code of 200 OK and a JSON body containing the session or an error
/// response if the session is not found.
///
/// # Errors
/// If an error occurs while retrieving the session, a session error response with a status code
/// of 404 Not Found is returned.
pub async fn get_session(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(session_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match get(read_lock, session_id).await {
        Ok(session) => Json(session).into_response(),
        Err(e) => SessionError::response(ApiStatusCode::from(StatusCode::NOT_FOUND), e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/sessions/add",
    request_body(
        content = inline(Session),
        description = "Session to add"
    ),
    responses(
        (status = 201, description = "Added session", body = ()),
        (status = 400, description = "Bad request", body = SessionError)
    )
)]
#[debug_handler]
/// Adds a new session.
///
/// This function is a handler for the route `POST /api/v1/sessions/add`. It adds a new session to the
/// database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `session` - The session to add
///
/// # Returns
/// `Response` with a status code of 201 Created and an empty body if the session was added or an
/// error response if the session could not be added.
///
/// # Errors
/// If an error occurs while adding the session, a session error response with a status code of 400
/// Bad Request is returned.
pub(crate) async fn post_session(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Extension(auth_info): Extension<AuthInfo>,
    Json(session): Json<Session>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match add(write_lock, session, auth_session, auth_info).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => SessionError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/sessions/add_for_user",
    request_body(
        content = inline(SessionAddedForUser),
        description = "Session to add"
    ),
    responses(
        (status = 201, description = "Added session", body = ()),
        (status = 400, description = "Bad request", body = SessionError)
    )
)]
#[debug_handler]
/// Adds a new session on behalf of a user.
///
/// This function is a handler for the route `POST /api/v1/sessions/add_for_user`. It adds allows
/// staff or admin to add a new session to the database on behalf of a user
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `session`: The `SessionAddedForUser` instance to add
/// - `auth_session`: Authentication session for authorization
///
/// # Returns
/// `Response` with a status code of 201 Created and an empty body if the session was added or an
/// error response if the session could not be added.
///
/// # Errors
/// If an error occurs while adding the session, a session error response with a status code of 400
/// Bad Request is returned.
pub(crate) async fn post_session_for_user(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Extension(auth_info): Extension<AuthInfo>,
    Json(session): Json<SessionAddedForUser>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match add_for_user(write_lock, session, auth_session, auth_info).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => SessionError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/sessions/{id}",
    responses(
        (status = 200, description = "Deleted session", body = ()),
        (status = 400, description = "Bad request", body = SessionError),
    )
)]
#[debug_handler]
/// Deletes a session
///
/// This function is a handler for the route `DELETE /api/v1/sessions/{id}`. It deletes a session from
/// the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `session_id` - The id of the session to delete
///
/// # Returns
/// `Response` with a status code of 200 OK if the session was deleted or an error response if the
/// session could not be deleted.
///
/// # Errors
/// If an error occurs while deleting the session, a session error response with a status code of
/// 400 Bad Request is returned.
pub(crate) async fn delete_session(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(session_id): Path<i32>,
    auth_session: AuthSessionLayer,
    Extension(auth_info): Extension<AuthInfo>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match delete(write_lock, session_id, auth_session, auth_info).await {
        Ok(()) => {
            let success_response = json!({
                "status": "success",
                "message": format!("Session {} successfully deleted", session_id)
            });
            (StatusCode::OK, Json(success_response)).into_response()
        }
        Err(e) => SessionError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/sessions/{id}",
    request_body(
        content = inline(Session),
        description = "Session to update"
    ),
    responses(
        (status = 200, description = "Updated session", body = ()),
        (status = 400, description = "Bad request", body = SessionError),
        (status = 404, description = "Session not found", body = SessionError),
        (status = 422, description = "Unprocessable entity", body = SessionError),
    )
)]
#[debug_handler]
/// Updates a session
///
/// This function is a handler for the route `PUT /api/v1/sessions/{id}`. It updates a session in the
/// database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `session_id` - The id of the session to update
/// - `session` - The session value to use for the update
///
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the session was updated or an error
/// response if the session could not be updated.
///
/// # Errors
/// If an error occurs while updating the session, a session error response with a status code of
/// 400 Bad Request is returned.
pub(crate) async fn update_session(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(session_id): Path<i32>,
    auth_session: AuthSessionLayer,
    Extension(auth_info): Extension<AuthInfo>,
    Json(session): Json<Session>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match update(write_lock, session_id, session, auth_session, auth_info).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => SessionError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}


