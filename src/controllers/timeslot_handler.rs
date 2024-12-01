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
use crate::config::AppState;

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
/// Updates a timeslot
/// 
/// This function is a handler for the route `PUT /api/v1/timeslot/{id}`. It updates a timeslot in
/// the database.
/// 
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `timeslot_id` - The id of the timeslot to update
/// - `timeslot` - The timeslot value to use for the update
/// 
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the timeslot was updated or an
/// error response if the timeslot could not be updated.
/// 
/// # Errors
/// This function returns a 400 error if:
/// - The timeslot could not be updated
/// - The timeslot does not exist
/// - The timeslot is invalid
pub async fn update_timeslot(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(timeslot_id): Path<i32>,
    Json(timeslot): Json<TimeSlot>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match timeslot_update(write_lock, timeslot_id, &timeslot).await {
        Ok(timeslot) => Json(timeslot).into_response(),
        Err(e) => TimeSlotError::response(StatusCode::BAD_REQUEST, e),
    }
}
