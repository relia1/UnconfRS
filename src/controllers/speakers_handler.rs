use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppState;
use crate::middleware::auth::AuthSessionLayer;
use crate::models::user_info_model::{
    user_info_add, user_info_delete, user_info_get, user_info_update, users_info_get, UserInfo, UserInfoErr,
    UserInfoError,
};
use crate::types::ApiStatusCode;
use crate::StatusCode;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tracing::trace;

#[utoipa::path(
    get,
    path = "/api/v1/speakers",
    params(
        ("page" = i32, Query, description = "Page", minimum = 1),
        ("limit" = i32, Query, description = "Limit", minimum = 1)
    ),
    responses(
        (status = 200, description = "List speakers", body = Vec<UserInfo>),
        (status = 404, description = "No speakers in that range")
    )
)]
#[debug_handler]
/// Retrieves a list of speakers
///
/// This function is a handler for the route `GET /api/v1/speakers`. It retrieves a list of speakers
/// from the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with a status code of 200 OK and a JSON body containing the list of speakers or an
/// error response if no speakers are found.
///
/// # Errors
/// If an error occurs while retrieving the speakers, a speaker error response with a status code
/// of 404 Not Found is returned.
pub async fn speakers(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match users_info_get(read_lock).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => {
            trace!("Paginated get error");
            UserInfoError::response(
                ApiStatusCode::from(StatusCode::NOT_FOUND),
                Box::new(UserInfoErr::DoesNotExist(e.to_string())),
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/speakers/{id}",
    responses(
        (status = 200, description = "Return specified speaker", body = UserInfo),
        (status = 404, description = "No speaker with this id", body = UserInfoError),
    )
)]
#[debug_handler]
/// Retrieves a speaker by id
///
/// This function is a handler for the route `GET /api/v1/speakers/{id}`. It retrieves a speaker
/// from the database by its id.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `speaker_id` - The id of the speaker to retrieve
///
/// # Returns
/// `Response` with a status code of 200 OK and a JSON body containing the speaker or an error
/// response if the speaker is not found.
///
/// # Errors
/// If an error occurs while retrieving the speaker, a speaker error response with a status code
/// of 404 Not Found is returned.
pub async fn get_speaker(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(speaker_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match user_info_get(read_lock, speaker_id).await {
        Ok(speaker) => Json(speaker).into_response(),
        Err(e) => UserInfoError::response(ApiStatusCode::from(StatusCode::NOT_FOUND), e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/speakers/add",
    request_body(
        content = inline(UserInfo),
        description = "UserInfo to add"
    ),
    responses(
        (status = 201, description = "Added speaker", body = ()),
        (status = 400, description = "Bad request", body = UserInfoError)
    )
)]
#[debug_handler]
/// Adds a new speaker.
///
/// This function is a handler for the route `POST /api/v1/speakers/add`. It adds a new speaker to
/// the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `speaker` - The speaker to add
///
/// # Returns
/// `Response` with a status code of 201 Created and an empty body if the speaker was added or an
/// error response if the speaker could not be added.
///
/// # Errors
/// If an error occurs while adding the speaker, a speaker error response with a status code of 400
pub async fn post_speaker(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Json(speaker): Json<UserInfo>,
) -> Response {
    tracing::trace!("\n\nauth_session: {:?}\n\n", auth_session.user);
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match user_info_add(write_lock, speaker, auth_session).await {
        Ok(id) => {
            let id_res = Json(format!("{{ \"id\": {id} }}"));
            id_res.into_response()
        }
        Err(e) => UserInfoError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/speakers/{id}",
    responses(
        (status = 200, description = "Deleted speaker", body = ()),
        (status = 400, description = "Bad request", body = UserInfoError),
    )
)]
#[debug_handler]
/// Deletes a speaker
///
/// This function is a handler for the route `DELETE /api/v1/speakers/{id}`. It deletes a speaker
/// from the database by its id.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `speaker_id` - The id of the speaker to delete
///
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the speaker was deleted or an error
/// response if the speaker could not be deleted.
///
/// # Errors
/// If an error occurs while deleting the speaker, a speaker error response with a status code of
/// 400 Bad Request is returned.
pub async fn delete_speaker(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(speaker_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match user_info_delete(write_lock, speaker_id).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => UserInfoError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/speakers/{id}",
    request_body(
        content = inline(UserInfo),
        description = "UserInfo to update"
    ),
    responses(
        (status = 200, description = "Updated speaker", body = ()),
        (status = 400, description = "Bad request", body = UserInfoError),
        (status = 404, description = "Speaker not found", body = UserInfoError),
        (status = 422, description = "Unprocessable entity", body = UserInfoError),
    )
)]
#[debug_handler]
/// Updates a speaker
///
/// This function is a handler for the route `PUT /api/v1/speakers/{id}`. It updates a speaker in
/// the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `speaker_id` - The id of the speaker to update
/// - `speaker` - The passed in speaker value to use for the update
///
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the speaker was updated or an error
/// response if the speaker could not be updated.
///
/// # Errors
/// If an error occurs while updating the speaker, a speaker error response with a status code of
/// 400 Bad Request is returned.
pub async fn update_speaker(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(speaker_id): Path<i32>,
    Json(speaker): Json<UserInfo>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match user_info_update(write_lock, speaker_id, speaker).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => UserInfoError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}
