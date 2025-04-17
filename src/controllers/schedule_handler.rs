use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppState;
use crate::controllers::site_handler::CreateScheduleForm;
use crate::models::schedule_model::{
    schedule_add, schedule_clear, schedule_generate, schedule_get, schedules_get, Schedule,
    ScheduleErr, ScheduleError,
};
use crate::types::ApiStatusCode;
use crate::StatusCode;
use askama_axum::IntoResponse;
use axum::debug_handler;
use axum::extract::State;
use axum::response::Response;
use axum::Json;
use tracing::trace;

#[utoipa::path(
    get,
    path = "/api/v1/schedules",
    responses(
        (status = 200, description = "List schedules", body = Vec<Schedule>),
    )
)]
#[debug_handler]
/// Retrieves a list of schedules
///
/// This function is a handler for the route `GET /api/v1/schedules`. It retrieves a list of
/// schedules from the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with a status code of 200 OK and a JSON body containing the list of schedules or an
/// error response if no schedules are found.
///
/// # Errors
/// If an error occurs while retrieving the schedules, a schedule error response with a status code
/// of 404 Not Found is returned.
pub async fn schedules(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match schedules_get(read_lock).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => {
            trace!("Paginated get error");
            ScheduleError::response(
                ApiStatusCode::from(StatusCode::NOT_FOUND),
                Box::new(ScheduleErr::DoesNotExist(e.to_string())),
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/schedules/{id}",
    responses(
        (status = 200, description = "Return specified schedule", body = Schedule),
        (status = 404, description = "No schedule with this id", body = ScheduleError),
    )
)]
#[debug_handler]
/// Retrieves a schedule by id
///
/// This function is a handler for the route `GET /api/v1/schedules/{id}`. It retrieves a schedule
/// from the database by its id.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `schedule_id` - The id of the schedule to retrieve
///
/// # Returns
/// `Response` with a status code of 200 OK and a JSON body containing the schedule or an error
/// response if the schedule is not found.
///
/// # Errors
/// If an error occurs while retrieving the schedule, a schedule error response with a status code
/// of 404 Not Found is returned.
pub async fn get_schedule(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match schedule_get(read_lock).await {
        Ok(schedule) => Json(schedule).into_response(),
        Err(e) => ScheduleError::response(ApiStatusCode::from(StatusCode::NOT_FOUND), e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/schedules/add",
    request_body(
        content = inline(Schedule),
        description = "Schedule to add"
    ),
    responses(
        (status = 201, description = "Added schedule", body = ()),
        (status = 400, description = "Bad request", body = ScheduleError)
    )
)]
#[debug_handler]
/// Adds a new schedule.
///
/// This function is a handler for the route `POST /api/v1/schedules/add`. It adds a new schedule to
/// the database.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `schedule_form` - The schedule form containing the schedule to add
///
/// # Returns
/// `Response` with a status code of 201 Created and an empty body if the schedule was added or an
/// error response if the schedule could not be added.
///
/// # Errors
/// If an error occurs while adding the schedule, a schedule error response with a status code of
/// 400 Bad Request is returned.
pub async fn post_schedule(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(schedule_form): Json<CreateScheduleForm>, //Json(schedule): Json<Schedule>,
) -> Response {
    tracing::info!("\n\nposting schedule!\n\n");
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match schedule_add(read_lock, Json(schedule_form)).await {
        Ok(id) => {
            trace!("id: {:?}\n", id);
            StatusCode::CREATED.into_response()
        }
        Err(e) => ScheduleError::response(ApiStatusCode::from(StatusCode::BAD_REQUEST), e),
    }
}

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
