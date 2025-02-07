use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppState;
use crate::models::{
    timeslot_assignment_model::{timeslot_assignment_swap, timeslot_assignment_update, TimeslotSwapRequest},
    timeslot_model::{timeslots_add, TimeSlot, TimeSlotError, TimeslotAssignmentForm, TimeslotForm, TimeslotRequest, TimeslotRequestWrapper, TimeslotUpdateRequest},
};
use askama_axum::IntoResponse;
use axum::debug_handler;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use chrono::NaiveTime;
use tracing::trace;

#[utoipa::path(
    post,
    path = "/api/v1/timeslots/add",
    request_body(
        content = inline(TimeslotRequest),
        description = "Timeslots to add"
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
/// This function is a handler for the route `POST /api/v1/timeslots`. It updates a timeslot in
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
pub async fn add_timeslots(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<TimeslotRequestWrapper>,
) -> Response {
    tracing::debug!("Before\nReceived request to add timeslot: {:?}", request);
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    tracing::debug!("Received request to add timeslot: {:?}", request);

    match timeslots_add(write_lock, request.timeslot_request).await {
        Ok(timeslot_ids) => Json(timeslot_ids).into_response(),
        Err(e) => {
            tracing::debug!("Error when trying to add timeslots: {:?}", e);
            TimeSlotError::response(StatusCode::INTERNAL_SERVER_ERROR.into(), e)
        }
    }
}

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
    Json(request): Json<TimeslotUpdateRequest>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;

    let start_time = match NaiveTime::parse_from_str(&request.start_time, "%H:%M") {
        Ok(time) => time,
        Err(e) => {
            trace!("Error parsing start time: {:?}", e);
            return TimeSlotError::response(StatusCode::BAD_REQUEST.into(), Box::new(e))
        },
    };

    let end_time = match NaiveTime::parse_from_str(&request.end_time, "%H:%M") {
        Ok(time) => time,
        Err(e) => return TimeSlotError::response(StatusCode::BAD_REQUEST.into(), Box::new(e)),
    };

    let duration = (end_time - start_time).num_minutes() as i32;

    let timeslot = TimeslotForm {
        start_time: request.start_time,
        duration,
        assignments: vec![TimeslotAssignmentForm {
            speaker_id: request.speaker_id,
            topic_id: request.topic_id,
            room_id: request.room_id,
            old_room_id: request.old_room_id,
        }],
    };

    match timeslot_assignment_update(write_lock, timeslot_id, TimeslotRequest { timeslots: vec![timeslot] }).await {
        Ok(assignment_ids) => Json(assignment_ids).into_response(),
        Err(e) => TimeSlotError::response(StatusCode::INTERNAL_SERVER_ERROR.into(), e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/timeslots/swap",
    request_body(
        content = inline(TimeslotSwapRequest),
        description = "Timeslots to swap assignments"
    ),
    responses(
        (status = 200, description = "Updated timeslots", body = ()),
        (status = 400, description = "Bad request", body = TimeSlotError),
        (status = 404, description = "Timeslot not found", body = TimeSlotError),
        (status = 422, description = "Unprocessable entity", body = TimeSlotError),
    )
)]
#[debug_handler]
/// Swaps 2 timeslots
///
/// This function is a handler for the route `PUT /api/v1/timeslots/swap`. It updates 2 timeslot in
/// the database swapping their assignments
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `timeslots composite key` - The values to identify the timeslots to swap
///
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the timeslots were updated or an
/// error response if the timeslots could not be updated.
///
/// # Errors
/// This function returns a 400 error if:
/// - The timeslots could not be updated
/// - The timeslot does not exist
/// - The timeslot is invalid
pub async fn swap_timeslots(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<TimeslotSwapRequest>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;

    match timeslot_assignment_swap(write_lock, request).await {
        Ok(_) => Json(()).into_response(),
        Err(e) => TimeSlotError::response(StatusCode::INTERNAL_SERVER_ERROR.into(), e),
    }
}