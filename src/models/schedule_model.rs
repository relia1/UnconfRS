use std::error::Error;
use crate::{
    models::{room_model::rooms_get, timeslot_model::*, topics_model::*},
    controllers::site_handler::CreateScheduleForm,
};
use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Response, Json};
use chrono::NaiveTime;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use tracing::debug;
use utoipa::{
    openapi::{ObjectBuilder, RefOr, Schema, SchemaType},
    ToSchema,
};

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
    #[error("Invalid query parameter values")]
    InvalidTimeFormat(String),
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

#[derive(Debug)]
/// Struct representing a `ScheduleError`
/// 
/// This struct represents an error that occurred while working with a schedule.
/// 
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
pub struct ScheduleError {
    pub status: StatusCode,
    pub error: String,
}

/// Implements the `ToSchema` trait for `ScheduleError`
/// 
/// This trait allows `ScheduleError` to be converted into a JSON schema.
impl<'s> ToSchema<'s> for ScheduleError {
    /// Returns a JSON schema for `ScheduleError`
    ///
    /// The schema defines two properties:
    /// - `status`: A string representing the HTTP status code associated with the error.
    /// - `error`: A string describing the specific error that occurred.
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
                "status":"404","error":"no schedule"
            })))
            .into();
        ("ScheduleError", sch)
    }
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
    pub fn response(status: StatusCode, error: Box<dyn Error>) -> Response {
        let error = ScheduleError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
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
    pub timeslots: Vec<TimeSlot>,
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
    pub fn new(id: Option<i32>, num_of_timeslots: i32, timeslots: Vec<TimeSlot>) -> Self {
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
    let mut schedules: Vec<Schedule> = sqlx::query_as::<Postgres, Schedule>(
        r#"
        SELECT * FROM schedules
        ORDER BY id"#,
    )
    .fetch_all(db_pool)
    .await?;

    for schedule in &mut schedules {
        let timeslots = sqlx::query_as::<Postgres, TimeSlot>(
            "SELECT * FROM time_slots WHERE schedule_id = $1 ORDER BY id;",
        )
        .bind(schedule.id)
        .fetch_all(db_pool)
        .await?;

        schedule.timeslots = timeslots;
    }

    Ok(schedules.first().cloned())
}

/// Retrieves a schedule by its index.
/// 
/// This function retrieves a schedule by its index.
/// 
/// # Parameters
/// - `db_pool` - The database connection pool
/// - `index` - The index of the schedule to retrieve
/// 
/// # Returns
/// A `Result` containing the `Schedule` or a `ScheduleErr` error.
/// 
/// # Errors
/// If an error occurs while fetching the schedule from the database, a `ScheduleErr` error is
/// returned.
pub async fn schedule_get(
    db_pool: &Pool<Postgres>,
    index: i32,
) -> Result<Schedule, Box<dyn Error>> {
    // Join the timeslots, schedules, topics, and speakers tables to get the full schedule data
    let schedule_vec = sqlx::query_as::<Postgres, Schedule>(
        r#"select ts.*, t.*, sched.*, s.* from time_slots ts
        join schedules sched on ts.schedule_id = sched.id
        left join topics t on t.id = ts.topic_id
        left join speakers s on ts.speaker_id = s.id
        where ts.schedule_id = $1
        group by ts.id, t.id, s.id, sched.id;"#,
    )
    .bind(index)
    .fetch_one(db_pool)
    .await?;

    Ok(schedule_vec)
}

/// Adds a new schedule.
///
/// This function adds a new schedule to the database.
/// 
/// # Parameters
/// - `db_pool` - The database connection pool
/// - `schedule_form` - The JSON body containing the schedule data
/// 
/// # Returns
/// A `Result` containing the `Schedule` or a `Box<dyn Error>` error.
/// 
/// # Errors
/// If an error occurs while adding the schedule to the database, a `Box<dyn Error>` error is
/// returned.
pub async fn schedule_add(
    db_pool: &Pool<Postgres>,
    Json(schedule_form): Json<CreateScheduleForm>,
) -> Result<Schedule, Box<dyn Error>> {
    let (schedule_id,) =
        sqlx::query_as(r#"INSERT INTO schedules (num_of_timeslots) VALUES ($1) RETURNING id"#)
            .bind(schedule_form.num_of_timeslots)
            .fetch_one(db_pool)
            .await?;

    let mut timeslots = vec![];
    for i in 0..(schedule_form.num_of_timeslots as usize) {
        let parse_time_from_string = |time| {
            NaiveTime::parse_from_str(time, "%H:%M").map_err(|error| {
                ScheduleErr::InvalidTimeFormat(
                    error.to_string(),
                )
            })
        };
        
        let start_time = parse_time_from_string(&schedule_form.start_time[i])?;
        let end_time = parse_time_from_string(&schedule_form.end_time[i])?;

        let mut timeslot = TimeSlot::new(
            None,
            start_time,
            end_time,
            None,
            Some(schedule_id),
            None,
            None,
        );
        let timeslot_id = timeslot_add(db_pool, timeslot.clone()).await?;
        timeslot.id = Some(timeslot_id);
        timeslots.push(timeslot);
    }

    Ok(Schedule::new(
        Some(schedule_id),
        schedule_form.num_of_timeslots,
        timeslots,
    ))
}

/// Updates a schedule.
/// 
/// This function updates a schedule in the database.
/// 
/// # Parameters
/// - `db_pool` - The database connection pool
/// - `index` - The index of the schedule to update
/// - `schedule` - The schedule data passed in to use for the update
/// 
/// # Returns
/// A `Result` containing the updated `Schedule` or a `Box<dyn Error>` error.
/// 
/// # Errors
/// If an error occurs while updating the schedule in the database, a `Box<dyn Error>` error is
/// returned.
pub async fn schedule_update(
    db_pool: &Pool<Postgres>,
    index: i32,
    schedule: Schedule,
) -> Result<Schedule, Box<dyn Error>> {
    // Update the schedule
    sqlx::query(
        r#"
        UPDATE schedules
        SET num_of_timeslots = $1
        WHERE id = $2
        "#,
    )
        .bind(schedule.num_of_timeslots)
        .bind(index)
        .execute(db_pool)
        .await?;

    // Update timeslots
    for timeslot in &schedule.timeslots {
        sqlx::query(
            r#"
            UPDATE time_slots
            SET
                start_time = $1,
                end_time = $2,
                duration = $3,
                speaker_id = $4,
                topic_id = $5,
                room_id = $6
            WHERE id = $7 AND schedule_id = $8
            "#,
        )
            .bind(timeslot.start_time)
            .bind(timeslot.end_time)
            .bind(timeslot.end_time - timeslot.start_time)
            .bind(timeslot.speaker_id)
            .bind(timeslot.topic_id)
            .bind(timeslot.room_id)
            .bind(timeslot.id)
            .bind(index)
            .execute(db_pool)
            .await?;
    }

    Ok(schedule)
}

/// Generates a schedule.
/// 
/// This function generates a schedule by assigning topics to timeslots.
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
    let topics = get_all_topics(db_pool).await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?;
    let rooms = rooms_get(db_pool).await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?
        .ok_or_else(|| ScheduleErr::DoesNotExist("No rooms found".to_string()))?;

    let mut schedule = schedules_get(db_pool).await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?
        .ok_or_else(|| ScheduleErr::DoesNotExist("No schedule found".to_string()))?;
    let schedule_id = schedule.id.ok_or_else(|| ScheduleErr::DoesNotExist("Schedule ID not found".to_string()))?;

    let existing_timeslots = timeslot_get(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?;

    let updated_timeslots = assign_topics_to_timeslots(&topics, &rooms, &existing_timeslots, schedule_id).await?;

    update_timeslots_in_db(db_pool, &updated_timeslots, schedule_id).await?;
    update_schedule_count(db_pool, schedule.num_of_timeslots, schedule_id).await?;

    schedule.timeslots = updated_timeslots;
    Ok(schedule)
}

/// Updates the number of timeslots in a schedule.
/// 
/// This function updates the number of timeslots in a schedule.
/// 
/// # Parameters
/// - `db_pool` - The database connection pool
/// - `num_of_timeslots` - The new number of timeslots
/// - `schedule_id` - The ID of the schedule to update
/// 
/// # Returns
/// A `Result` containing `()` or a `ScheduleErr` error.
/// 
/// # Errors
/// If an error occurs while updating the number of timeslots in the schedule, a `ScheduleErr` error
/// is returned.
async fn update_schedule_count(
    db_pool: &Pool<Postgres>,
    num_of_timeslots: i32,
    schedule_id: i32,
) -> Result<(), ScheduleErr> {
    sqlx::query("UPDATE schedules SET num_of_timeslots = $1 WHERE id = $2")
        .bind(num_of_timeslots)
        .bind(schedule_id)
        .execute(db_pool)
        .await
        .map_err(|e| ScheduleErr::IoError(e.to_string()))?;
    Ok(())
}

/// Clears the schedule by removing topic associations with timeslots.
/// 
/// This function clears the schedule by removing topic associations with timeslots.
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
    let schedule = schedules_get(db_pool).await?.ok_or("No schedule found")?;
    let schedule_id = schedule.id.ok_or("Schedule ID not found")?;

    sqlx::query(
        r#"
        UPDATE time_slots
        SET
            speaker_id = NULL,
            topic_id = NULL
        WHERE
            topic_id IS NOT NULL
            AND speaker_id IS NOT NULL
            AND schedule_id = $1
        "#,
    )
    .bind(schedule_id)
    .execute(db_pool)
    .await?;

    Ok(())
}
