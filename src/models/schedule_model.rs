use crate::models::room_model::RoomErr;
use crate::models::timeslot_assignment_model::{assign_sessions_to_timeslots, get_all_unassigned_timeslots, session_already_scheduled, space_to_add_session};
use crate::models::{room_model::rooms_get, sessions_model::{get_all_sessions, SessionErr}, timeslot_model::{timeslot_get, ExistingTimeslot}};
use crate::types::ApiStatusCode;
use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Response, Json};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use std::error::Error;
use utoipa::ToSchema;

/// An enumeration of errors that may occur
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
/// An enumeration of `Schedule` errors that may occur
///
/// This enum represents the different types of errors that may occur when working with schedules.
///
/// # Variants
/// - `IoError` - An I/O error occurred
/// - `DoesNotExist` - The schedule does not exist
/// - `InvalidTimeFormat` - The time format is invalid
pub enum ScheduleErr {
    #[error("Schedule io failed: {0}")]
    IoError(String),
    #[error("Schedule {0} doesn't exist")]
    DoesNotExist(String),
    #[error("Session error: {0}")]
    SessionError(SessionErr),
    #[error("Room error: {0}")]
    RoomError(RoomErr),
    #[error("Session {0} already scheduled")]
    SessionAlreadyScheduled(String),
    #[error("No space to add session {0}")]
    ScheduleFull(String),
}

/// Implements the `From` trait for `std::io::Error` to convert it into a `ScheduleErr`.
///
/// This allows `std::io::Error` instances to be converted into `ScheduleErr`, wrapping the I/O
/// error as a `ScheduleIoError`.
impl From<std::io::Error> for ScheduleErr {
    /// Converts a `std::io::Error` into a `ScheduleErr`.
    ///
    /// This function converts a `std::io::Error` into a `ScheduleErr` by wrapping the error message
    /// in a `ScheduleIoError`.
    ///
    /// # Parameters
    /// - `e` - The `std::io::Error` to convert
    ///
    /// # Returns
    /// A `ScheduleErr` with the error message from the `std::io::Error`.
    fn from(e: std::io::Error) -> Self {
        ScheduleErr::IoError(e.to_string())
    }
}

impl From<SessionErr> for ScheduleErr {
    fn from(err: SessionErr) -> Self {
        ScheduleErr::SessionError(err)
    }
}

impl From<RoomErr> for ScheduleErr {
    fn from(err: RoomErr) -> Self {
        ScheduleErr::RoomError(err)
    }
}

#[derive(Debug, ToSchema)]
/// Struct representing a `ScheduleError`
///
/// This struct represents an error that occurred while working with a schedule.
///
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
pub struct ScheduleError {
    pub status: ApiStatusCode,
    pub error: String,
}

/// Implements the `Serialize` trait for `ScheduleError`
///
/// This trait allows `ScheduleError` to be serialized into JSON.
impl Serialize for ScheduleError {
    /// Serializes a `ScheduleError`
    ///
    /// The serialized JSON object will have two properties:
    /// - `status`: A string for the HTTP status code
    /// - `error`: A string describing the error
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status: String = self.status.to_string();
        let mut state = serializer.serialize_struct("ScheduleError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl ScheduleError {
    /// Creates a `Response` instance from a `StatusCode` and `ScheduleErr`.
    ///
    /// This function creates a `Response` instance from a `StatusCode` and a `ScheduleErr`.
    ///
    /// # Parameters
    /// - `status` - The HTTP status code to return
    /// - `error` - The `ScheduleErr`
    ///
    /// # Returns
    /// A `Response` instance with the specified status code and error.
    pub fn response(status: ApiStatusCode, error: Box<dyn Error>) -> Response {
        let error = ScheduleError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
}

#[derive(Deserialize, ToSchema)]
pub struct AddSessionReq {
    pub session_id: i32,
}


#[derive(Deserialize, ToSchema)]
pub struct RemoveSessionReq {
    pub session_id: i32,
    pub timeslot_id: i32,
    pub room_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
/// Struct representing a `Schedule`
///
/// This struct represents a schedule with a unique ID, number of timeslots, and a list of
/// timeslots.
///
/// # Fields
/// - `id` - The unique ID of the schedule
/// - `num_of_timeslots` - The number of timeslots in the schedule
/// - `timeslots` - A list of timeslots in the schedule
pub struct Schedule {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub num_of_timeslots: i32,
    #[sqlx(skip)]
    pub timeslots: Vec<ExistingTimeslot>,
}

impl Schedule {
    /// Creates a new `Schedule` instance.
    ///
    /// This function creates a new `Schedule` instance with the specified ID, number of timeslots,
    /// and list of timeslots.
    ///
    /// # Parameters
    /// - `id` - The unique ID of the schedule
    /// - `num_of_timeslots` - The number of timeslots in the schedule
    /// - `timeslots` - A list of timeslots in the schedule
    ///
    /// # Returns
    /// A new `Schedule` instance
    pub fn new(id: Option<i32>, num_of_timeslots: i32, timeslots: Vec<ExistingTimeslot>) -> Self {
        Self {
            id,
            num_of_timeslots,
            timeslots,
        }
    }
}

/// Implements the `IntoResponse` trait for `&Schedule`
///
/// This trait allows a reference to a `Schedule` to be converted into an HTTP response.
impl IntoResponse for &Schedule {
    /// Converts a `&Schedule` into an HTTP response.
    ///
    /// # Returns
    /// A `Response` object with a status code of 200 OK and a JSON body containing the schedule
    /// data.
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self)).into_response()
    }
}

/// Retrieves a paginated list of schedules from the schedule .
///
/// This function retrieves a paginated list of schedules from the schedule.
///
/// # Parameters
/// - `db_pool` - The database connection pool
///
/// # Returns
/// A `Result` containing a `Option<Schedule>` or a `ScheduleErr` error.
///
/// # Errors
/// If an error occurs while fetching the schedules from the database, a `ScheduleErr` error is
/// returned.
pub async fn schedules_get(db_pool: &Pool<Postgres>) -> Result<Option<Schedule>, Box<dyn Error>> {
    let timeslots = timeslot_get(db_pool).await?;
    if timeslots.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Schedule::new(
            Some(1),
            i32::try_from(timeslots.len())?,
            timeslots,
        )))
    }
}


/// Generates a schedule.
///
/// This function generates a schedule by assigning sessions to timeslots.
///
/// # Parameters
/// - `db_pool` - The database connection pool
///
/// # Returns
/// A `Result` containing the generated `Schedule` or a `ScheduleErr` error.
///
/// # Errors
/// If an error occurs while generating the schedule, a `ScheduleErr` error is returned.
pub async fn schedule_generate(db_pool: &Pool<Postgres>) -> Result<Schedule, ScheduleErr> {
    let sessions = get_all_sessions(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?;
    let rooms = rooms_get(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?
        .ok_or_else(|| ScheduleErr::DoesNotExist("No rooms found".to_string()))?;
    let mut schedule = schedules_get(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?
        .ok_or_else(|| ScheduleErr::DoesNotExist("No schedule found".to_string()))?;

    let existing_timeslots = timeslot_get(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?;

    match assign_sessions_to_timeslots(&sessions, &rooms, &existing_timeslots, db_pool).await {
        Ok(_) => {
            schedule.timeslots = timeslot_get(db_pool)
                .await
                .map_err(|e| ScheduleErr::IoError(e.to_string()))?;

            Ok(schedule)
        }
        Err(e) => Err(ScheduleErr::IoError(e.to_string())),
    }
}

/// Generates a schedule.
///
/// This function generates a schedule by assigning sessions to timeslots.
///
/// # Parameters
/// - `db_pool` - The database connection pool
///
/// # Returns
/// A `Result` containing the generated `Schedule` or a `ScheduleErr` error.
///
/// # Errors
/// If an error occurs while generating the schedule, a `ScheduleErr` error is returned.
pub async fn add_session(db_pool: &Pool<Postgres>, session_id: i32) -> Result<Schedule, ScheduleErr> {
    if session_already_scheduled(db_pool, session_id).await? {
        return Err(ScheduleErr::SessionAlreadyScheduled(session_id.to_string()));
    }

    if !space_to_add_session(db_pool).await? {
        return Err(ScheduleErr::ScheduleFull(session_id.to_string()));
    }

    let unassigned_timeslots = get_all_unassigned_timeslots(db_pool).await?;

    if let Some(first_timeslot) = unassigned_timeslots.first() {
        sqlx::query!(
            "INSERT INTO timeslot_assignments (time_slot_id, session_id, room_id) VALUES ($1, $2, $3)",
            first_timeslot.time_slot_id,
            session_id,
            first_timeslot.room_id,
        )
            .execute(db_pool)
            .await
            .map_err(|e| ScheduleErr::IoError(e.to_string()))?;

        let timeslots = timeslot_get(db_pool)
            .await
            .map_err(|e| ScheduleErr::IoError(e.to_string()))?;

        Ok(Schedule::new(
            Some(1),
            i32::try_from(timeslots.len()).map_err(|e| ScheduleErr::IoError(e.to_string()))?,
            timeslots,
        ))
    } else {
        Err(ScheduleErr::IoError(session_id.to_string()))
    }
}

pub async fn remove_session(
    db_pool: &Pool<Postgres>,
    session_id: i32,
    timeslot_id: i32,
    room_id: i32,
) -> Result<Schedule, ScheduleErr> {
    let affected_rows = sqlx::query!(
        "DELETE FROM timeslot_assignments
        WHERE session_id = $1 AND time_slot_id = $2 AND room_id = $3",
        session_id,
        timeslot_id,
        room_id,
    )
        .execute(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?
        .rows_affected();

    if affected_rows == 0 {
        return Err(ScheduleErr::DoesNotExist(format!(
            "Session {} not found in time {} room {}",
            session_id, timeslot_id, room_id,
        )));
    }

    let timeslots = timeslot_get(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?;

    Ok(Schedule::new(
        Some(1),
        i32::try_from(timeslots.len()).map_err(|e| ScheduleErr::IoError(e.to_string()))?,
        timeslots,
    ))
}

/// Clears the schedule by removing session associations with timeslots.
///
/// This function clears the schedule by removing session associations with timeslots.
///
/// # Parameters
/// - `db_pool` - The database connection pool
///
/// # Returns
/// A `Result` containing `()` or a `Box<dyn Error>` error.
///
/// # Errors
/// If an error occurs while clearing the schedule, a `Box<dyn Error>` error is returned.
pub async fn schedule_clear(db_pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
    sqlx::query!(r"DELETE FROM timeslot_assignments")
        .execute(db_pool)
        .await?;

    Ok(())
}
