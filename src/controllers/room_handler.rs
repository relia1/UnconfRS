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
use tracing::trace;
use utoipa::OpenApi;

use crate::UnconfData;

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
pub async fn rooms(
    State(db_pool): State<Arc<RwLock<UnconfData>>>,
) -> Response {
    let read_lock = db_pool.read().await;
    match rooms_get(&read_lock.unconf_db).await {
        Ok(res) => {
            Json(res).into_response()
        }
        Err(e) => {
            RoomError::response(
                StatusCode::NOT_FOUND,
                Box::new(RoomErr::DoesNotExist(e.to_string())),
            )
        }
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
pub async fn post_rooms(
    State(db_pool): State<Arc<RwLock<UnconfData>>>,
    Json(rooms_form): Json<CreateRoomsForm>
) -> Response {
    tracing::info!("\n\nposting rooms!\n\n");
    let write_lock = db_pool.write().await;
    match rooms_add(&write_lock.unconf_db, Json(rooms_form)).await {
        Ok(id) => {
            trace!("id: {:?}\n", id);
            StatusCode::CREATED.into_response()
        },
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
    State(db_pool): State<Arc<RwLock<UnconfData>>>,
    Path(room_id): Path<i32>,
) -> Response {
    tracing::info!("delete room");
    let write_lock = db_pool.write().await;
    match room_delete(&write_lock.unconf_db, room_id).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => RoomError::response(StatusCode::BAD_REQUEST, e),
    }
}
