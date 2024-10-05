use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::timeslot_model::{timeslot_update, TimeSlot, TimeSlotError};
use crate::StatusCode;
use askama_axum::IntoResponse;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Response;
use axum::Json;
use tracing::trace;
use utoipa::OpenApi;
use crate::config::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        update_timeslot,
    ),
    components(
        schemas(TimeSlot, TimeSlotError)
    ),
    tags(
        (name = "Schedules Server API", description = "Schedules Server API")
    )
)]
pub struct ApiDocTimeslot;

#[utoipa::path(
    put,
    path = "/api/v1/timeslot/{id}",
    request_body(
        content = inline(TimeSlot),
        description = "Timeslot to update"
    ),
    responses(
        (status = 200, description = "Updated timeslot", body = ()),
        (status = 400, description = "Bad request", body = TimeSlotError),
        (status = 404, description = "Timeslot not found", body = TimeSlotError),
        (status = 422, description = "Unprocessable entity", body = TimeSlotError),
    )
)]
#[debug_handler]
pub async fn update_timeslot(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(timeslot_id): Path<i32>,
    Json(timeslot): Json<TimeSlot>,
) -> Response {
    trace!("timeslot id: {:?}", &timeslot.id);
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match timeslot_update(write_lock, timeslot_id, &timeslot).await {
        Ok(timeslot) => Json(timeslot).into_response(),
        Err(e) => TimeSlotError::response(StatusCode::BAD_REQUEST, e),
    }
}
