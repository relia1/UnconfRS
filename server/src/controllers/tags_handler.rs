use crate::config::AppState;
use crate::middleware::auth::AuthSessionLayer;
use crate::models::tags_model::{self, Tag, TagError};
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
pub struct CreateTagRequest {
    pub tag_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTagRequest {
    pub tag_name: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/tags",
    responses(
        (status = 200, description = "List of all available tags", body = [Tag]),
    )
)]
#[debug_handler]
/// Gets all available tags
///
/// This function is a handler for the route `GET /api/v1/tags`.
/// It retrieves all available tags.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with a status code of 200 OK and a JSON array of all tags.
///
/// # Errors
/// If an error occurs while retrieving tags, an error response is returned.
pub async fn get_all_tags(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let db_pool = &app_state_lock.unconf_data.read().await.unconf_db;

    match tags_model::get_all_tags(db_pool).await {
        Ok(tags) => (StatusCode::OK, Json(tags)).into_response(),
        Err(e) => TagError::response(ApiStatusCode::from(StatusCode::INTERNAL_SERVER_ERROR), e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/tags/{tag_id}",
    responses(
        (status = 200, description = "Tag details", body = Tag),
        (status = 404, description = "Tag not found", body = TagError),
    )
)]
#[debug_handler]
/// Gets a specific tag by ID
///
/// This function is a handler for the route `GET /api/v1/tags/{tag_id}`.
/// It gets a tag by its ID.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `tag_id` - The id of the tag to get
///
/// # Returns
/// `Response` with a status code of 200 OK and the tag data, or 404 if not found.
///
/// # Errors
/// If the tag is not found or an error occurs, an error response is returned.
pub async fn get_tag_by_id(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(tag_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let db_pool = &app_state_lock.unconf_data.read().await.unconf_db;

    match tags_model::get_tag_by_id(db_pool, tag_id).await {
        Ok(tag) => (StatusCode::OK, Json(tag)).into_response(),
        Err(e) => TagError::response(ApiStatusCode::from(StatusCode::NOT_FOUND), e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/tags",
    request_body = CreateTagRequest,
    responses(
        (status = 201, description = "Tag created", body = Tag),
        (status = 409, description = "Tag already exists", body = TagError),
        (status = 403, description = "Unauthorized", body = TagError),
    )
)]
#[debug_handler]
/// Creates a new tag
///
/// This function is a handler for the route `POST /api/v1/tags`.
/// It creates a new tag in the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `auth_session` - Authentication session for authorization
/// - `request` - JSON body containing the tag name
///
/// # Returns
/// `Response` with a status code of 201 Created and the new tag data.
///
/// # Errors
/// If the tag already exists, user is unauthorized, or an error occurs,
/// an error response is returned.
pub async fn create_tag(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Json(request): Json<CreateTagRequest>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let db_pool = &app_state_lock.unconf_data.read().await.unconf_db;

    match tags_model::create_tag(db_pool, auth_session, &request.tag_name).await {
        Ok(tag) => (StatusCode::CREATED, Json(tag)).into_response(),
        Err(e) => {
            let status = if e.to_string().contains("already exists") {
                StatusCode::CONFLICT
            } else if e.to_string().contains("does not have access") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::BAD_REQUEST
            };
            TagError::response(ApiStatusCode::from(status), e)
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/tags/{tag_id}",
    request_body = UpdateTagRequest,
    responses(
        (status = 200, description = "Tag updated", body = Tag),
        (status = 404, description = "Tag not found", body = TagError),
        (status = 409, description = "Tag name already exists", body = TagError),
        (status = 403, description = "Unauthorized", body = TagError),
    )
)]
#[debug_handler]
/// Updates an existing tag
///
/// This function is a handler for the route `PUT /api/v1/tags/{tag_id}`.
/// It updates the name of an existing tag.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `auth_session` - Authentication session for authorization
/// - `tag_id` - The id of the tag to update
/// - `request` - JSON body containing the new tag name
///
/// # Returns
/// `Response` with a status code of 200 OK and the updated tag data.
///
/// # Errors
/// If the tag is not found, new name already exists, user is unauthorized,
/// or an error occurs, an error response is returned.
pub async fn update_tag(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Path(tag_id): Path<i32>,
    Json(request): Json<UpdateTagRequest>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let db_pool = &app_state_lock.unconf_data.read().await.unconf_db;

    match tags_model::update_tag(db_pool, auth_session, tag_id, &request.tag_name).await {
        Ok(tag) => (StatusCode::OK, Json(tag)).into_response(),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("already exists") {
                StatusCode::CONFLICT
            } else if e.to_string().contains("does not have access") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::BAD_REQUEST
            };
            TagError::response(ApiStatusCode::from(status), e)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/tags/{tag_id}",
    responses(
        (status = 204, description = "Tag deleted"),
        (status = 404, description = "Tag not found", body = TagError),
        (status = 403, description = "Unauthorized", body = TagError),
    )
)]
#[debug_handler]
/// Deletes a tag
///
/// This function is a handler for the route `DELETE /api/v1/tags/{tag_id}`.
/// It deletes a tag from the database. This will also remove the tag from
/// all sessions due to CASCADE DELETE.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `auth_session` - Authentication session for authorization
/// - `tag_id` - The id of the tag to delete
///
/// # Returns
/// `Response` with a status code of 204 No Content if successful.
///
/// # Errors
/// If the tag is not found, user is unauthorized, or an error occurs,
/// an error response is returned.
pub async fn delete_tag(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Path(tag_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let db_pool = &app_state_lock.unconf_data.read().await.unconf_db;

    match tags_model::delete_tag(db_pool, auth_session, tag_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("does not have access") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::BAD_REQUEST
            };
            TagError::response(ApiStatusCode::from(status), e)
        }
    }
}