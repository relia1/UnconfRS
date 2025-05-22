use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppState;
use crate::models::schedule_model::{
    schedule_clear, schedule_generate,
    ScheduleError,
};
use crate::types::ApiStatusCode;
use axum::{debug_handler, extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};

#[utoipa::path(
    post,
    path = "/api/v1/schedules/generate",
    responses(
        (status = 200, description = "Generating schedule", body = ()),
        (status = 400, description = "Bad request", body = ScheduleError),
        (status = 404, description = "Schedule not found", body = ScheduleError),
        (status = 422, description = "Unprocessable entity", body = ScheduleError),
    )
)]
#[debug_handler]
/// Generates a schedule
///
/// This function is a handler for the route `POST /api/v1/schedules/generate`. It generates a
/// schedule based on the data in the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the schedule was generated or an
/// error response if the schedule could not be generated.
///
/// # Errors
/// If an error occurs while generating the schedule, a schedule error response with a status code
/// of 400 Bad Request is returned.
pub async fn generate(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    let res = schedule_generate(read_lock).await;
    match res {
        Ok(schedule) => Json(schedule).into_response(),
        Err(e) => {
            ScheduleError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), Box::new(e))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/schedules/clear",
    responses(
        (status = 200, description = "Clearing schedule", body = ()),
        (status = 400, description = "Bad request", body = ScheduleError),
        (status = 404, description = "Schedule not found", body = ScheduleError),
        (status = 422, description = "Unprocessable entity", body = ScheduleError),
    )
)]
#[debug_handler]
/// Clears a schedule
///
/// This function is a handler for the route `POST /api/v1/schedules/clear`. It clears the schedule
/// data in the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the schedule was cleared or an
/// error response if the schedule could not be cleared.
///
/// # Errors
/// If an error occurs while clearing the schedule, a schedule error response with a status code
/// of 400 Bad Request is returned.
pub async fn clear(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    let res = schedule_clear(read_lock).await;
    match res {
        Ok(schedule) => Json(schedule).into_response(),
        Err(e) => ScheduleError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}
