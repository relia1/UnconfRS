use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::schedule_model::{
    schedule_add, schedule_clear, schedule_generate, schedule_get,
    schedule_update, schedules_get, Schedule, ScheduleErr, ScheduleError,
};
use crate::models::timeslot_model::TimeSlot;
use crate::CreateScheduleForm;
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
use crate::controllers::topics_handler::topics;

#[derive(OpenApi)]
#[openapi(
    paths(
        schedules,
        get_schedule,
        post_schedule,
        update_schedule,
        generate,
    ),
    components(
        schemas(Schedule, ScheduleError, TimeSlot)
    ),
    tags(
        (name = "Schedules Server API", description = "Schedules Server API")
    )
)]
pub struct ApiDocSchedule;

#[utoipa::path(
    get,
    path = "/api/v1/schedules",
    responses(
        (status = 200, description = "List schedules", body = Vec<Schedule>),
    )
)]
pub async fn schedules(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match schedules_get(read_lock).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => {
            trace!("Paginated get error");
            ScheduleError::response(
                StatusCode::NOT_FOUND,
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
pub async fn get_schedule(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(schedule_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match schedule_get(read_lock, schedule_id).await {
        Ok(schedule) => Json(schedule).into_response(),
        Err(e) => ScheduleError::response(StatusCode::NOT_FOUND, e),
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
        Err(e) => ScheduleError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/schedules/{id}",
    request_body(
        content = inline(Schedule),
        description = "Schedule to update"
    ),
    responses(
        (status = 200, description = "Updated schedule", body = ()),
        (status = 400, description = "Bad request", body = ScheduleError),
        (status = 404, description = "Schedule not found", body = ScheduleError),
        (status = 422, description = "Unprocessable entity", body = ScheduleError),
    )
)]
#[debug_handler]
pub async fn update_schedule(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(schedule_id): Path<i32>,
    Json(schedule): Json<Schedule>,
) -> Response {
    trace!("schedule id: {:?}", &schedule.id);
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match schedule_update(read_lock, schedule_id, schedule).await {
        Ok(schedule) => Json(schedule).into_response(),
        Err(e) => ScheduleError::response(StatusCode::BAD_REQUEST, e),
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
pub async fn generate(State(app_state): State<Arc<RwLock<AppState>>>,) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    let res = schedule_generate(read_lock).await;
    match res {
        Ok(schedule) => {
            Json(schedule).into_response()
            //StatusCode::OK.into_response()
        }
        Err(e) => ScheduleError::response(StatusCode::BAD_REQUEST, e),
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
pub async fn clear(State(app_state): State<Arc<RwLock<AppState>>>,) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    let res = schedule_clear(read_lock).await;
    match res {
        Ok(schedule) => {
            Json(schedule).into_response()
            //StatusCode::OK.into_response()
        }
        Err(e) => ScheduleError::response(StatusCode::BAD_REQUEST, e),
    }
}
