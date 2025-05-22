use crate::config::AppState;
use crate::middleware::auth::{AuthInfo, AuthSessionLayer};
use crate::models::auth_model::Permission;
use crate::models::room_model::{rooms_get, Room};
use crate::models::schedule_model::{schedules_get, Schedule};
use crate::models::session_voting_model::get_sessions_user_voted_for;
use crate::models::sessions_model::get_all_sessions;
use crate::models::timeslot_model::{timeslot_get, ExistingTimeslot, TimeslotAssignment};
use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Extension;
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

#[debug_handler]
/// Fall back handler
///
/// This function is a handler for requests that do not match any other route.
///
/// # Returns
/// `Response` with a status code of 404 Not Found.
pub async fn handler_404() -> Response {
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

#[derive(Template, Debug)]
#[template(path = "index.html")]
/// Index template
struct IndexTemplate {
    is_authenticated: bool,
    permissions: HashSet<Permission>,
}

#[debug_handler]
/// Index handler
///
/// This function renders the index page.
///
/// # Returns
/// `Response` with the rendered HTML page or an error status code.
///
/// # Errors
/// If the template fails to render, an internal server error status code is returned.
pub(crate) async fn index_handler(Extension(auth_info): Extension<AuthInfo>) -> Response {
    let is_authenticated = auth_info.is_authenticated;
    let permissions = auth_info.permissions;
    let template = IndexTemplate { is_authenticated, permissions };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

#[derive(Debug, Serialize)]
/// Event struct
///
/// This struct represents the parameters of an event.
///
/// # Fields
/// - `timeslot_id` - The ID of the timeslot
/// - `title` - The title of the event
/// - `start_time` - The start time of the event
/// - `end_time` - The end time of the event
/// - `room_id` - The ID of the room
/// - `session_id` - The ID of the session
/// - `schedule_id` - The ID of the schedule
pub struct Event {
    pub timeslot_id: i32,
    pub title: String,
    pub start_time: String,
    pub end_time: String,
    pub room_id: i32,
    pub session_id: i32,
    pub schedule_id: i32,
}

#[derive(Template, Debug, Serialize)]
#[template(path = "create_schedule.html")]
pub(crate) struct ScheduleTemplate {
    pub(crate) schedule: Option<Schedule>,
    pub(crate) rooms: Option<Vec<Room>>,
    pub(crate) events: Vec<Event>,
    is_authenticated: bool,
    permissions: HashSet<Permission>,
}

#[debug_handler]
/// Schedule handler
///
/// This function renders the schedule page.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with the rendered HTML page or an error status code.
///
/// # Errors
/// If the template fails to render, an internal server error status code is returned.
pub(crate) async fn schedule_handler(State(app_state): State<Arc<RwLock<AppState>>>, Extension(auth_info): Extension<AuthInfo>) -> Response {
    tracing::info!("Schedule handler");
    let is_authenticated = auth_info.is_authenticated;
    let permissions = auth_info.permissions;
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;

    let result: Result<String, Response> = async {
        let schedule = schedules_get(read_lock)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

        let rooms = rooms_get(read_lock).await.unwrap_or(None);
        let sessions = get_all_sessions(read_lock).await.unwrap_or_default();
        let timeslots = timeslot_get(read_lock)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
        let assignments =
            sqlx::query_as::<_, TimeslotAssignment>("SELECT * FROM timeslot_assignments")
                .fetch_all(read_lock)
                .await
                .unwrap_or_default();

        let events = if let Some(schedule) = &schedule {
            let schedule_id = schedule
                .id
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
            timeslots
                .iter()
                .flat_map(|timeslot| {
                    let timeslot_id = timeslot.id;

                    assignments
                        .iter()
                        .filter(|assignment| assignment.time_slot_id == timeslot_id)
                        .filter_map(|filtered_assignment| {
                            let event_session = sessions
                                .iter()
                                .find(|&session| session.id == Some(filtered_assignment.session_id))?;

                            Some(Event {
                                timeslot_id,
                                title: event_session.title.clone(),
                                start_time: timeslot.start_time.to_string(),
                                end_time: timeslot.end_time.to_string(),
                                room_id: filtered_assignment.room_id,
                                session_id: filtered_assignment.session_id,
                                schedule_id,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        } else {
            vec![]
        };

        let template = ScheduleTemplate {
            schedule,
            rooms,
            events,
            is_authenticated,
            permissions
        };

        template
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
        .await;

    match result {
        Ok(html) => Html(html).into_response(),
        Err(response) => response,
    }
}
#[derive(Template, Debug)]
#[template(path = "sessions.html")]
/// Sessions template
///
/// This struct represents the parameters passed to the client for rendering the sessions page.
///
/// # Fields
/// - `sessions` - A vector containing the sessions and users
struct SessionsTemplate {
    sessions: Vec<SessionAndUser>,
    current_users_voted_sessions: Vec<i32>,
    is_authenticated: bool,
    permissions: HashSet<Permission>,
}

#[derive(Debug, Deserialize, FromRow)]
/// `Session` and `User` struct
///
/// This struct represents the pairing of a `Session` and `User`.
///
/// # Fields
/// - `session` - The session `Session`
/// - `user_info` - The `User` for the `Session`
pub struct SessionAndUser {
    pub session_id: i32,
    pub title: String,
    pub content: String,
    pub user_id: i32,
    pub fname: String,
    pub lname: String,
    pub email: String,
}


/// Combined session and user query
///
/// This function queries the database for a combination of sessions and users.
///
/// # Parameters
/// - `db_pool` - The database connection pool
///
/// # Returns
/// A vector containing the sessions and users or an error if the query fails.
///
/// # Errors
/// An error is returned if the query fails.
pub async fn combine_session_and_user(
    db_pool: &Pool<Postgres>,
) -> Result<Vec<SessionAndUser>, Box<dyn Error>> {
    let session_with_user: Vec<SessionAndUser> = sqlx::query_as::<Postgres, SessionAndUser>(
        "SELECT t.id as \"session_id\", t.title, t.content, \
        u.id as \"user_id\", u.fname, u.lname, u.email \
        FROM sessions t \
        JOIN users u ON u.id = t.user_id",
    )
        .fetch_all(db_pool)
        .await?;

    Ok(session_with_user)
}

#[debug_handler]
/// Session handler
///
/// This function renders the sessions page.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with the rendered HTML page or an error status code.
///
/// # Errors
/// If the template fails to render, an internal server error status code is returned.
pub(crate) async fn session_handler(
    State(app_state): State<Arc<RwLock<AppState>>>,
    auth_session: AuthSessionLayer,
    Extension(auth_info): Extension<AuthInfo>
) -> Response {
    let is_authenticated = auth_info.is_authenticated;
    let permissions = auth_info.permissions;
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    let current_users_voted_sessions = if let Some(user) = auth_session.user.clone() {
        get_sessions_user_voted_for(write_lock, user.id).await.unwrap_or(Vec::<i32>::new())
    } else {
        Vec::<i32>::new()
    };
    let sessions_with_user_info = combine_session_and_user(write_lock).await;


    match sessions_with_user_info {
        Ok(sessions_and_users) => {
            let template = SessionsTemplate {
                sessions: sessions_and_users,
                current_users_voted_sessions,
                is_authenticated,
                permissions,
            };

            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(e) => {
            tracing::debug!(e);
            Html("<h1>Error fetching sessions</h1>".to_string()).into_response()
        }
    }
}

#[derive(Template, Clone, Deserialize, FromRow)]
#[template(path = "unconf_timeslots.html")]
struct UnconfTimeslotsTemplate {
    existing_timeslots: Vec<ExistingTimeslot>,
    is_authenticated: bool,
    permissions: HashSet<Permission>,
}

#[debug_handler]
pub(crate) async fn unconf_timeslots_handler(State(app_state): State<Arc<RwLock<AppState>>>, Extension(auth_info): Extension<AuthInfo>) -> Response {
    let is_authenticated = auth_info.is_authenticated;
    let permissions = auth_info.permissions;
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;

    let timeslots = match timeslot_get(read_lock).await {
        Ok(timeslots) => timeslots,
        Err(e) => {
            tracing::error!("Error getting timeslots: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    tracing::debug!("Timeslots: {:?}", timeslots);

    let template = UnconfTimeslotsTemplate {
        existing_timeslots: timeslots,
        is_authenticated,
        permissions,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Error rendering template: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}


#[derive(Template, Clone, Deserialize, FromRow)]
#[template(path = "config.html")]
struct ConfigTemplate {
    permissions: HashSet<Permission>,
    is_authenticated: bool,
}
#[debug_handler]
pub(crate) async fn config_handler(State(app_state): State<Arc<RwLock<AppState>>>, Extension(auth_info): Extension<AuthInfo>) -> Response {
    let app_state_lock = app_state.read().await;
    let _read_lock = &app_state_lock.unconf_data.read().await.unconf_db;

    let template = ConfigTemplate {
        permissions: auth_info.permissions,
        is_authenticated: auth_info.is_authenticated,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Error rendering template: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}