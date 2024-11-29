use axum::extract::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::room_model::{room_delete, rooms_add, rooms_get, Room, RoomErr, RoomError};
use crate::CreateRoomsForm;
use crate::StatusCode;
use askama_axum::IntoResponse;
use axum::extract::State;
use axum::response::Response;
use axum::Json;
use axum_macros::debug_handler;
use tracing::trace;
use utoipa::OpenApi;
use crate::config::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        rooms,
        post_rooms,
        delete_room,
    ),
    components(
        schemas(Room)
    ),
    tags(
        (name = "Rooms API", description = "Rooms API")
    )
)]
pub struct ApiDocRooms;

#[utoipa::path(
    get,
    path = "/api/v1/rooms",
    responses(
        (status = 200, description = "List rooms", body = Vec<Room>),
    )
)]
pub async fn rooms(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match rooms_get(read_lock).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => RoomError::response(
            StatusCode::NOT_FOUND,
            Box::new(RoomErr::DoesNotExist(e.to_string())),
        ),
    }
}

#[debug_handler]
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
pub async fn post_rooms(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(rooms_form): Json<CreateRoomsForm>,
) -> impl IntoResponse {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match rooms_add(write_lock, rooms_form).await {
        Ok(schedule) => {
            trace!("Schedule created: {:?}", schedule);
            (StatusCode::CREATED, Json(schedule)).into_response()
        }
        Err(e) => RoomError::response(StatusCode::BAD_REQUEST, e),
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
pub async fn delete_room(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(room_id): Path<i32>,
) -> Response {
    tracing::info!("delete room");
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match room_delete(write_lock, room_id).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => RoomError::response(StatusCode::BAD_REQUEST, e),
    }
}
