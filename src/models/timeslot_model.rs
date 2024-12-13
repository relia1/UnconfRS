use std::error::Error;
use askama_axum::IntoResponse;
use axum::{response::Response, Json};
use chrono::NaiveTime;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use utoipa::ToSchema;
use crate::models::room_model::Room;
use crate::models::schedule_model::ScheduleErr;
use crate::models::topics_model::Topic;
use crate::types::ApiStatusCode;

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

/// Implements the `ToSchema` trait for `TimeSlotError` struct.
/// 
/// This implementation provides a JSON schema for the `TimeSlotError` struct. The schema defines 
/// two properties: `status` and `error`.

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

#[derive(Debug, Deserialize)]
pub struct TimeSlotData {
    pub start_time: String,
    pub duration: i32,
}

#[derive(Debug, Deserialize)]
pub struct TimeslotForm {
    pub timeslots: Vec<TimeSlotData>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
/// Struct that represents a timeslot.
/// 
/// # Fields
/// - `Option<id>` - The ID of the timeslot (optional)
/// - `start_time` - The start time of the timeslot
/// - `end_time` - The end time of the timeslot
/// - `Option<speaker_id>` - The ID of the speaker for the timeslot (optional)
/// - `Option<schedule_id>` - The ID of the schedule for the timeslot (optional)
/// - `Option<topic_id>` - The ID of the topic for the timeslot (optional)
/// - `Option<room_id>` - The ID of the room for the timeslot (optional)
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
    /// Creates a new `TimeSlot` instance.
    /// 
    /// This function creates a new `TimeSlot` instance with the provided ID, start time, end time,
    /// speaker ID, schedule ID, topic ID, and room ID.
    /// 
    /// # Parameters
    /// - `id`: The ID of the timeslot (optional)
    /// - `start_time`: The start time of the timeslot
    /// - `end_time`: The end time of the timeslot
    /// - `speaker_id`: The ID of the speaker for the timeslot (optional)
    /// - `schedule_id`: The ID of the schedule for the timeslot (optional)
    /// - `topic_id`: The ID of the topic for the timeslot (optional)
    /// - `room_id`: The ID of the room for the timeslot (optional)
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
pub async fn timeslot_get(db_pool: &Pool<Postgres>) -> Result<Vec<TimeSlot>, Box<dyn Error>> {
    let timeslots = sqlx::query_as("SELECT * FROM time_slots")
        .fetch_all(db_pool)
        .await?;

    Ok(timeslots)
}

/// Assigns topics to timeslots.
/// 
/// This function assigns topics to timeslots based on the provided topics, rooms, and existing
/// timeslots. The topics are assigned to the timeslots in the order they are provided, starting
/// with the first topic and moving to the next topic for each room.
/// 
/// # Parameters
/// - `topics`: A slice of `Topic` instances representing the topics to assign
/// - `rooms`: A slice of `Room` instances representing the rooms to assign the topics to
/// - `existing_timeslots`: A slice of `TimeSlot` instances representing the existing timeslots
/// - `schedule_id`: The ID of the schedule to assign the timeslots to
/// 
/// # Returns
/// A `Result` containing a vector of `TimeSlot` instances with the topics assigned if successful,
/// otherwise a `ScheduleErr` error.
/// 
/// # Errors
/// If an I/O error occurs, a `ScheduleErr` error is returned.
pub async fn assign_topics_to_timeslots(
    topics: &[Topic],
    rooms: &[Room],
    existing_timeslots: &[TimeSlot],
    schedule_id: i32,
) -> Result<Vec<TimeSlot>, ScheduleErr> {
    let mut result = Vec::new();
    let mut topic_index = 0;

    for room in rooms {
        let room_timeslots: Vec<_> = existing_timeslots.iter()
            .filter(|slot| slot.room_id == room.id)
            .collect();

        for slot in room_timeslots {
            if topic_index >= topics.len() {
                break;
            }

            let topic = &topics[topic_index];
            let updated_slot = TimeSlot::new(
                slot.id,
                slot.start_time,
                slot.end_time,
                Some(topic.speaker_id),
                Some(schedule_id),
                topic.id,
                room.id,
            );

            result.push(updated_slot);
            topic_index += 1;
        }
        
        if topic_index >= topics.len() {
            break;
        }
    }

    Ok(result)
}

/// Adds a new timeslot.
///
/// This function adds a new timeslot to the database.
/// 
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `timeslot`: The timeslot to add
/// 
/// # Returns
/// The ID of the timeslot if successful, otherwise an error.
/// 
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn timeslot_add(
    db_pool: &Pool<Postgres>,
    timeslot: TimeSlot,
) -> Result<i32, Box<dyn Error>> {
    let (timeslot_id,) = sqlx::query_as(
        r#"INSERT INTO time_slots (start_time, end_time, duration, speaker_id, schedule_id, 
        topic_id, room_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"#)
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

/// Adds a new timeslot.
///
/// This function adds a new timeslot to the database.
///
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `timeslot`: The timeslot to add
///
/// # Returns
/// The ID of the timeslot if successful, otherwise an error.
///
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn timeslots_add(
    db_pool: &Pool<Postgres>,
    timeslots: TimeslotForm,
) -> Result<(), Box<dyn Error>> {
    for timeslot in timeslots.timeslots {
        let start_time = NaiveTime::parse_from_str(&timeslot.start_time, "%H:%M:%S")?;
        let end_time = start_time + chrono::Duration::minutes(timeslot.duration as i64);
        sqlx::query_as(
            r#"INSERT INTO time_slots (start_time, end_time, duration, speaker_id, schedule_id,
        topic_id, room_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"#)
            .bind(timeslot.start_time)
            .bind(end_time)
            .bind(timeslot.duration)
            .bind(1)
            .bind(1)
            .bind(1)
            .bind(1)
            .fetch_one(db_pool)
            .await?
    }

    Ok(())
}

/// Updates a timeslot by its ID.
///
/// This function updates a timeslot by its ID.
/// 
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `timeslot_id`: The ID of the timeslot to update
/// - `timeslot`: The timeslot to use for the update
/// 
/// # Returns
/// The ID of the updated timeslot if successful, otherwise an error.
/// 
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn timeslot_update(
    db_pool: &Pool<Postgres>,
    timeslot_id: i32,
    timeslot: &TimeSlot,
) -> Result<i32, Box<dyn Error>> {
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

/// Updates a timeslot by its ID.
/// 
/// This function updates a timeslot by its ID.
/// 
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `timeslot_id`: The ID of the timeslot to update
/// - `timeslot`: The timeslot to use for the update
/// 
/// # Returns
/// The ID of the updated timeslot if successful, otherwise an error.
/// 
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn update_timeslots_in_db(
    db_pool: &Pool<Postgres>, 
    timeslots: &[TimeSlot], 
    schedule_id: i32
) -> Result<(), ScheduleErr> {
    for slot in timeslots {
        sqlx::query(
            r#"UPDATE time_slots
            SET start_time = $1, end_time = $2, duration = $3, 
                speaker_id = $4, topic_id = $5, room_id = $6
            WHERE id = $7 AND schedule_id = $8"#,
        )
            .bind(slot.start_time)
            .bind(slot.end_time)
            .bind(slot.end_time - slot.start_time)
            .bind(slot.speaker_id)
            .bind(slot.topic_id)
            .bind(slot.room_id)
            .bind(slot.id)
            .bind(schedule_id)
            .execute(db_pool)
            .await
            .map_err(|e| ScheduleErr::IoError(e.to_string()))?;
    }
    Ok(())
}
