use axum::extract::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppState;
use crate::models::room_model::{
    room_delete, rooms_add, rooms_get, CreateRoomsForm, Room, RoomErr, RoomError,
};
use crate::types::ApiStatusCode;
use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};
use axum_macros::debug_handler;
use tracing::debug;

#[utoipa::path(
    get,
    path = "/api/v1/rooms",
    responses(
        (status = 200, description = "List rooms", body = Vec<Room>),
        (status = 404, description = "No rooms found", body = RoomError)
    )
)]
#[debug_handler]
/// Retrieves a list of rooms
///
/// This function is a handler for the route `GET /api/v1/rooms`. It retrieves a list of rooms from
/// the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with a status code of 200 OK and a JSON body containing the list of rooms. If no
/// rooms are found, a room error response with a status code of 404 Not Found is returned.
///
/// # Errors
/// If an error occurs while retrieving the rooms, a room error response with a status code of 404
pub async fn rooms(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match rooms_get(read_lock).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => RoomError::response(
            ApiStatusCode::from(StatusCode::NOT_FOUND),
            Box::new(RoomErr::DoesNotExist(e.to_string())),
        ),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/rooms/add",
    request_body(
        content = inline(Room),
        description = "Rooms to add"
    ),
    responses(
        (status = 201, description = "Added room", body = ()),
        (status = 400, description = "Bad request", body = RoomError)
    )
)]
#[debug_handler]
/// Adds new rooms.
///
/// This function is a handler for the route `POST /api/v1/rooms/add`. It adds new rooms to the
/// database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `rooms_form` - The rooms form containing the rooms to add
///
/// # Returns
/// `Response` with a status code of 201 Created and an empty body if the rooms were added. If an
/// error occurs while adding the rooms, a room error response with a status code of 400 Bad Request
/// is returned.
///
/// # Errors
/// If an error occurs while adding the rooms, a room error response with a status code of 400
/// Bad Request is returned.
pub(crate) async fn post_rooms(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(rooms_form): Json<CreateRoomsForm>,
) -> impl IntoResponse {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match rooms_add(write_lock, rooms_form).await {
        Ok(schedule) => {
            debug!("Schedule created: {:?}", schedule);
            (StatusCode::CREATED, Json(schedule)).into_response()
        }
        Err(e) => RoomError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/rooms/{id}",
    responses(
        (status = 200, description = "Deleted room", body = ()),
        (status = 400, description = "Bad request", body = RoomError),
    )
)]
#[debug_handler]
/// Deletes a room.
///
/// This function is a handler for the route `DELETE /api/v1/rooms/{id}`. It deletes a room from the
/// database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `room_id` - The ID of the room to delete
///
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the room was deleted. If an error
/// occurs while deleting the room, a room error response with a status code of 400 Bad Request is
/// returned.
///
/// # Errors
/// If an error occurs while deleting the room, a room error response with a status code of 400
/// Bad Request is returned.
pub async fn delete_room(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(room_id): Path<i32>,
) -> Response {
    tracing::info!("delete room");
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match room_delete(write_lock, room_id).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => RoomError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}
