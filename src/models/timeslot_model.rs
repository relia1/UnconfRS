use std::error::Error;
use std::process::id;
use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Response, Json};
use chrono::NaiveTime;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use utoipa::{
    openapi::{ObjectBuilder, RefOr, Schema, SchemaType},
    ToSchema,
};

/// An enumeration of errors that may occur
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum TimeSlotErr {
    #[error("TimeSlot io failed: {0}")]
    IoError(String),
}

impl From<std::io::Error> for TimeSlotErr {
    /// Converts a `std::io::Error` into a `TimeSlotErr`.
    ///
    /// # Description
    ///
    /// This allows `std::io::Error` instances to be converted into
    /// `TimeSlotErr`, wrapping the I/O error as a `TimeSlotIoError`.
    ///
    /// # Example
    ///
    /// ```
    /// let io_err = std::io::Error::new(std::io::ErrorKind::Other, "IO error");
    /// let timeslot_err: TimeSlotErr = io_err.into();
    /// ```
    fn from(e: std::io::Error) -> Self {
        TimeSlotErr::IoError(e.to_string())
    }
}

/// struct that represents a TimeSlot error, but include a `StatusCode`
/// in addition to a `TimeSlotErr`
#[derive(Debug)]
pub struct TimeSlotError {
    pub status: StatusCode,
    pub error: String,
}

/// Implements `ToSchema` trait for `TimeSlotError` generating a JSON schema
/// for the error type
impl<'s> ToSchema<'s> for TimeSlotError {
    /// Returns a JSON schema for `TimeSlotError`
    ///
    /// The schema defines two properties:
    ///
    /// * `status`: A string representing the HTTP status code associated with the error.
    /// * `error`: A string describing the specific error that occurred.
    fn schema() -> (&'s str, RefOr<Schema>) {
        let sch = ObjectBuilder::new()
            .property(
                "status",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .property(
                "error",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .example(Some(serde_json::json!({
                "status":"404","error":"no timeslot"
            })))
            .into();
        ("TimeSlotError", sch)
    }
}

/// Implements the `Serialize` trait for `TimeSlotError`
impl Serialize for TimeSlotError {
    /// Serializes a `TimeSlotError`
    ///
    /// The serialized JSON object will have two properties:
    ///
    /// * `status`: A string for the HTTP status code
    /// * `error`: A string describing the error
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
    /// # Parameters
    ///
    /// * `status`: The HTTP status code.
    /// * `error`: The `TimeSlotErr` instance.
    ///
    /// # Returns
    ///
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: StatusCode, error: Box<dyn Error>) -> Response {
        let error = TimeSlotError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TimeSlot {
    pub id: Option<i32>,
    pub start_time: NaiveTime,    // unix timestamp (seconds since epoch)
    pub end_time: NaiveTime,      // unix timestamp (seconds since epoch)
    pub speaker_id: Option<i32>,  // id of the speaker
    pub schedule_id: Option<i32>, // id of the schedule
    pub topic_id: Option<i32>,    // id of the topic or none
    pub room_id: Option<i32>,     // id of the topic or none
}

impl TimeSlot {
    pub fn new(
        id: Option<i32>,
        start_time: NaiveTime,
        end_time: NaiveTime,
        speaker_id: Option<i32>,
        schedule_id: Option<i32>,
        topic_id: Option<i32>,
        room_id: Option<i32>,
    ) -> Self {
        Self {
            id,
            start_time,
            end_time,
            speaker_id,
            schedule_id,
            topic_id,
            room_id,
        }
    }
}

/// Adds a new timeslot.
///
/// # Parameters
///
/// * `timeslot`: The `TimeSlot` to add to the timeslot .
///
/// # Returns
///
/// A `Result` indicating whether the timeslot was added successfully.
/// If the timeslot already exists, returns a `TimeSlotErr` error.
pub async fn timeslot_add(
    db_pool: &Pool<Postgres>,
    timeslot: TimeSlot,
) -> Result<i32, Box<dyn Error>> {
    let (timeslot_id,) = sqlx::query_as(r#"INSERT INTO time_slots (start_time, end_time, 
    duration, speaker_id, schedule_id, topic_id, room_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"#)
        .bind(timeslot.start_time)
        .bind(timeslot.end_time)
        .bind(timeslot.end_time - timeslot.start_time)
        .bind(timeslot.speaker_id)
        .bind(timeslot.schedule_id)
        .bind(timeslot.topic_id)
        .bind(timeslot.room_id)
        .fetch_one(db_pool)
        .await?;

    Ok(timeslot_id)
}

/// Updates a timeslot by its ID.
///
/// # Parameters
///
/// * `index`: The ID of the timeslot.
///
/// # Returns
///
/// A `Result` indicating whether the timeslot was removed successfully.
/// If the timeslot does not exist, returns a `TimeSlotErr` error.
pub async fn timeslot_update(
    db_pool: &Pool<Postgres>,
    timeslot_id: i32,
    timeslot: &TimeSlot,
) -> Result<i32, Box<dyn Error>> {
    tracing::trace!("updating timeslot id: {}\nstart time: {}\nend time: {}\nspeaker id: \
    {}\nschedule id: {}\ntopic id: {}\nroom id: {}",
                    timeslot_id, timeslot.start_time, timeslot.end_time, timeslot.speaker_id
        .unwrap(), 
        timeslot.schedule_id.unwrap(), timeslot.topic_id.unwrap(), timeslot.room_id.unwrap());
    let (new_timeslot_id,) = sqlx::query_as(
        r#"
        UPDATE time_slots SET start_time = $1, end_time = $2, speaker_id = $3, schedule_id = $4,
        topic_id = $5, room_id = $6
        WHERE start_time = $1 AND room_id = $6
        RETURNING id
        "#,
    )
        .bind(timeslot.start_time)
        .bind(timeslot.end_time)
        .bind(timeslot.speaker_id)
        .bind(timeslot.schedule_id)
        .bind(timeslot.topic_id.unwrap())
        .bind(timeslot.room_id)
        .fetch_one(db_pool)
        .await?;

    sqlx::query(
        r#"
        UPDATE time_slots SET speaker_id = NULL, topic_id = NULL
        WHERE id = $1
        "#,
    )
        .bind(timeslot_id)
        .execute(db_pool)
        .await?;

    Ok(new_timeslot_id)
}
