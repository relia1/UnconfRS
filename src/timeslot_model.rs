use std::error::Error;


use askama_axum::IntoResponse;
use axum::{http::StatusCode, Json, response::Response};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres, Row};
use utoipa::{openapi::{ObjectBuilder, RefOr, Schema, SchemaType}, ToSchema};
use chrono::NaiveTime;

/// An enumeration of errors that may occur
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum TimeSlotErr {
    #[error("TimeSlot io failed: {0}")]
    IoError(String),
    #[error("Invalid query parameter values")]
    PaginationInvalid(String),
}

/*
#[derive(OpenApi)]
#[openapi(
    paths(
        ,
        get_topic,
        post_topic,
        delete_topic,
        update_topic,
    ),
    components(
        schemas(Topic, TopicError)
    ),
    tags(
        (name = "Topics Server API", description = "Topics Server API")
    )
)]
pub struct ApiDocTimeslot;
*/

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
    pub start_time: NaiveTime, // unix timestamp (seconds since epoch)
    pub end_time: NaiveTime, // unix timestamp (seconds since epoch)
    pub speaker_id: Option<i32>, // id of the speaker
    pub schedule_id: Option<i32>, // id of the schedule
    pub topic_id: Option<i32>, // id of the topic or none
    pub room_id: Option<i32> // id of the topic or none
}

impl TimeSlot {
    pub fn new(
        id: Option<i32>,
        start_time: NaiveTime,
        end_time: NaiveTime,
        speaker_id: Option<i32>,
        schedule_id: Option<i32>,
        topic_id: Option<i32>,
        room_id: Option<i32>
    )
    -> Self {
        Self {
            id,
            start_time,
            end_time,
            speaker_id,
            schedule_id,
            topic_id,
            room_id
        }
    }
}

/// Retrieves a list of timeslots.
///
/// # Parameters
///
/// # Returns
///
/// A vector of TimeSlot's
pub async fn timeslots_get(
    db_pool: &Pool<Postgres>,
) -> Result<Vec<TimeSlot>, Box<dyn Error>> {
    let timeslots: Vec<TimeSlot> = sqlx::query_as(
        r#"
        SELECT * FROM time_slots
        ORDER BY id"#
    )
        .fetch_all(db_pool)
        .await?;

    Ok(timeslots)
}


/// Retrieves a list of timeslots.
///
/// # Parameters
///
/// * `timeslots`: Pooled db connection
///
/// # Returns
///
/// A vector of TimeSlot's
/// If the pagination parameters are invalid, returns a `TimeSlotErr` error.
pub async fn get_all_timeslots(
    db_pool: &Pool<Postgres>,
) -> Result<Vec<TimeSlot>, Box<dyn Error>> {
    let timeslots: Vec<TimeSlot> = sqlx::query_as(
        r#"
        SELECT * FROM time_slots
        ORDER BY id"#
    )
        .fetch_all(db_pool)
        .await?;

    Ok(timeslots)
}


/// Retrieves a timeslot by its ID.
///
/// # Parameters
///
/// * `index`: The ID of the timeslot.
///
/// # Returns
///
/// A reference to the `TimeSlot` instance with the specified ID, or a `TimeSlotErr` error if the timeslot does not exist.
pub async fn timeslot_get(db_pool: &Pool<Postgres>, index: i32) -> Result<Vec<TimeSlot>, Box<dyn Error>> {
    let timeslots: Vec<_> = sqlx::query_as::<Postgres, TimeSlot>(
        "SELECT *
        FROM time_slots
        WHERE id = $1;",
    )
    .bind(index)
    .fetch_all(db_pool)
    .await?;

    //timeslot_vec.push(<TimeSlot as std::convert::From<PgRow>>::from(timeslot));
    Ok(timeslots)
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
pub async fn timeslot_add(db_pool: &Pool<Postgres>, timeslot: TimeSlot) -> Result<i32, Box<dyn Error>> {
    let timeslot_id: (i32,) = sqlx::query_as(r#"INSERT INTO time_slots (start_time, end_time, duration, speaker_id, schedule_id, topic_id, room_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"#)
        .bind(timeslot.start_time)
        .bind(timeslot.end_time)
        .bind(timeslot.end_time - timeslot.start_time)
        .bind(timeslot.speaker_id)
        .bind(timeslot.schedule_id)
        .bind(timeslot.topic_id)
        .bind(timeslot.room_id)
        .fetch_one(db_pool)
        .await?;

    Ok(timeslot_id.0)
}

/// Removes a timeslot by its ID.
///
/// # Parameters
///
/// * `index`: The ID of the timeslot.
///
/// # Returns
///
/// A `Result` indicating whether the timeslot was removed successfully.
/// If the timeslot does not exist, returns a `TimeSlotErr` error.
pub async fn timeslot_delete(db_pool: &Pool<Postgres>, index: i32) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        DELETE FROM timeslots
        WHERE id = $1
        "#,
    )
    .bind(index)
    .execute(db_pool)
    .await?;

    Ok(())
}

/// Removes a timeslot by its ID.
///
/// # Parameters
///
/// * `index`: The ID of the timeslot.
///
/// # Returns
///
/// A `Result` indicating whether the timeslot was removed successfully.
/// If the timeslot does not exist, returns a `TimeSlotErr` error.
pub async fn timeslot_update(db_pool: &Pool<Postgres>, timeslot_id: i32, timeslot: &TimeSlot) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        UPDATE time_slots SET start_time = $2, end_time = $3, speaker_id = $4, schedule_id = $5, topic_id = $6, room_id = $7
        WHERE id = $1
        "#,
    )
        .bind(timeslot_id)
        .bind(timeslot.start_time)
        .bind(timeslot.end_time)
        .bind(timeslot.speaker_id)
        .bind(timeslot.schedule_id)
        .bind(timeslot.topic_id.unwrap())
        .bind(timeslot.room_id)
        .execute(db_pool)
        .await?;

    Ok(())
}
