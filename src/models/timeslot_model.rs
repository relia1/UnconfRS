use crate::types::ApiStatusCode;
use axum::response::IntoResponse;
use axum::{response::Response, Json};
use chrono::NaiveTime;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use std::error::Error;
use utoipa::ToSchema;

type BoxedError = Box<dyn Error + Send + Sync>;

/// An enumeration of possible errors that can occur when working with timeslots.
///
/// # Variants
/// - `IoError` - An I/O error occurred
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum TimeSlotErr {
    #[error("TimeSlot io failed: {0}")]
    IoError(String),
}

/// Implements the `From` trait for `std::io::Error` to convert it into a `TimeSlotErr`.
///
/// This implementation allows `std::io::Error` instances to be converted into `TimeSlotErr`
/// instances. The I/O error is wrapped as a `TimeSlotIoError`.
impl From<std::io::Error> for TimeSlotErr {
    /// Converts a `std::io::Error` into a `TimeSlotErr`.
    ///
    /// This allows `std::io::Error` instances to be converted into
    /// `TimeSlotErr`, wrapping the I/O error as a `TimeSlotIoError`.
    ///
    /// # Example
    fn from(e: std::io::Error) -> Self {
        TimeSlotErr::IoError(e.to_string())
    }
}

/// Struct that represents an error that occurred when working with timeslots.
///
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
#[derive(Debug, ToSchema)]
pub struct TimeSlotError {
    pub status: ApiStatusCode,
    pub error: String,
}

/// Implements the `Serialize` trait for `TimeSlotError`
///
/// This implementation serializes a `TimeSlotError` into a JSON object with two properties:
/// `status` and `error`.
impl Serialize for TimeSlotError {
    /// Serializes a `TimeSlotError`
    ///
    /// The serialized JSON object will have two properties:
    /// - `status`: A string for the HTTP status code
    /// - `error`: A string describing the error
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status: String = self.status.to_string();
        let mut state = serializer.serialize_struct("TimeSlotError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl TimeSlotError {
    /// Creates a `Response` instance from a `StatusCode` and `TimeSlotErr`.
    ///
    /// This function creates a `Response` instance from a `StatusCode` and a `TimeSlotErr`. The
    /// `TimeSlotErr` is serialized into a JSON object with two properties: `status` and `error`.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code to return
    /// - `error`: The `TimeSlotErr` to return
    ///
    /// # Returns
    /// A `Response` instance with the HTTP status code and the serialized `TimeSlotErr`.
    pub fn response(status: ApiStatusCode, error: Box<dyn Error>) -> Response {
        let error = TimeSlotError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimeslotAssignmentForm {
    pub session_id: i32,
    pub room_id: i32,
    pub old_room_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimeslotForm {
    pub start_time: String,
    pub duration: i32,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub assignments: Vec<TimeslotAssignmentForm>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimeslotRequest {
    pub timeslots: Vec<TimeslotForm>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimeslotRequestWrapper {
    pub timeslot_request: TimeslotRequest,
}

#[derive(Debug, Deserialize)]
pub struct TimeslotUpdateRequest {
    pub start_time: String,
    pub end_time: String,
    pub session_id: i32,
    pub room_id: i32,
    pub old_room_id: i32,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TimeSlot {
    pub id: Option<i32>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct ExistingTimeslot {
    pub id: i32,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub duration: i32,
}

#[derive(Debug, Deserialize, FromRow, Clone)]
pub struct TimeslotAssignment {
    pub time_slot_id: i32,
    pub session_id: i32,
    pub room_id: i32,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct TimeslotAssignmentSessionAdd {
    pub time_slot_id: i32,
    pub session_id: Option<i32>,
    pub room_id: i32,
}

/// Retrieves all timeslots from the database.
///
/// This function retrieves all timeslots from the database.
///
/// # Parameters
/// - `db_pool`: The database connection pool
///
/// # Returns
/// A `Result` containing a vector of `TimeSlot` instances if successful, otherwise an error.
///
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn timeslot_get(
    db_pool: &Pool<Postgres>,
) -> Result<Vec<ExistingTimeslot>, Box<dyn Error>> {
    let timeslots = sqlx::query_as!(
        ExistingTimeslot,
        r#"SELECT id, start_time as "start_time!: NaiveTime", end_time as "end_time!: NaiveTime",
        (EXTRACT(EPOCH FROM duration) / 60)::integer as "duration!"
        FROM time_slots"#,
    )
        .fetch_all(db_pool)
        .await?;

    tracing::debug!("Timeslots: {:?}", timeslots);

    Ok(timeslots)
}

async fn insert_timeslot(
    db_pool: &Pool<Postgres>,
    start_time: NaiveTime,
    duration: i64,
) -> Result<i32, Box<dyn Error>> {
    let end_time = start_time + chrono::Duration::minutes(duration);
    let duration_interval = format!("{duration} minutes");
    let id = sqlx::query_scalar!(
        "INSERT INTO time_slots (start_time, end_time, duration) VALUES ($1, $2, $3::interval) RETURNING id",
        start_time as _,
        end_time as _,
        duration_interval as _,
    )
        .fetch_one(db_pool)
        .await?;
    Ok(id)
}

/// Adds new timeslots.
///
/// This function adds new timeslots to the database.
///
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `timeslots`: The timeslots to add
///
/// # Returns
/// Vec of IDs of the timeslots if successful, otherwise an error.
///
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn timeslots_add(
    db_pool: &Pool<Postgres>,
    timeslots: TimeslotRequest,
) -> Result<Vec<i32>, Box<dyn Error>> {
    tracing::debug!("Adding timeslots: {:?}", timeslots);
    let mut timeslot_ids = Vec::new();
    for timeslot in timeslots.timeslots {
        let start_time = NaiveTime::parse_from_str(&timeslot.start_time, "%H:%M")?;
        let id = insert_timeslot(db_pool, start_time, i64::from(timeslot.duration)).await?;
        if !timeslot.assignments.is_empty() {
            tracing::debug!("Adding assignments: {:?}", timeslot.assignments);
            //insert_assignments(db_pool, id, timeslot.assignments).await?;
        }
        timeslot_ids.push(id);
    }
    Ok(timeslot_ids)
}

pub async fn get_num_timeslots(db_pool: &Pool<Postgres>) -> Result<i32, BoxedError> {
    let num_timeslots = sqlx::query_scalar!("SELECT COUNT(*)::INTEGER FROM time_slots")
        .fetch_one(db_pool)
        .await
        .map_err(|e| Box::new(e) as BoxedError)?;

    // This is safe to unwrap since it should always return a number
    Ok(num_timeslots.unwrap())
}