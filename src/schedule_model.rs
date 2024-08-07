use std::error::Error;


use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Response, Form, Json};
use chrono::NaiveTime;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{Pool, Postgres, FromRow};
use tracing::trace;
use utoipa::{openapi::{ObjectBuilder, RefOr, Schema, SchemaType}, ToSchema};

use crate::{timeslot_model::*, topics_model::get_all_topics, CreateScheduleForm};


/// An enumeration of errors that may occur
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum ScheduleErr {
    #[error("Schedule io failed: {0}")]
    IoError(String),
    #[error("Schedule {0} doesn't exist")]
    DoesNotExist(String),
    #[error("Invalid query parameter values")]
    PaginationInvalid(String),
    #[error("Invalid query parameter values")]
    InvalidTimeFormat(String),

}

impl From<std::io::Error> for ScheduleErr {
    /// Converts a `std::io::Error` into a `ScheduleErr`.
    ///
    /// # Description
    ///
    /// This allows `std::io::Error` instances to be converted into
    /// `ScheduleErr`, wrapping the I/O error as a `ScheduleIoError`.
    ///
    /// # Example
    ///
    /// ```
    /// let io_err = std::io::Error::new(std::io::ErrorKind::Other, "IO error");
    /// let schedule_err: ScheduleErr = io_err.into();
    /// ```
    fn from(e: std::io::Error) -> Self {
        ScheduleErr::IoError(e.to_string())
    }
}

/// struct that represents a Schedule error, but include a `StatusCode`
/// in addition to a `ScheduleErr`
#[derive(Debug)]
pub struct ScheduleError {
    pub status: StatusCode,
    pub error: String,
}

/// Implements `ToSchema` trait for `ScheduleError` generating a JSON schema
/// for the error type
impl<'s> ToSchema<'s> for ScheduleError {
    /// Returns a JSON schema for `ScheduleError`
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
                "status":"404","error":"no schedule"
            })))
            .into();
        ("ScheduleError", sch)
    }
}

/// Implements the `Serialize` trait for `ScheduleError`
impl Serialize for ScheduleError {
    /// Serializes a `ScheduleError`
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
        let mut state = serializer.serialize_struct("ScheduleError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl ScheduleError {
    /// Creates a `Response` instance from a `StatusCode` and `ScheduleErr`.
    ///
    /// # Parameters
    ///
    /// * `status`: The HTTP status code.
    /// * `error`: The `ScheduleErr` instance.
    ///
    /// # Returns
    ///
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: StatusCode, error: Box<dyn Error>) -> Response {
        let error = ScheduleError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
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
    /// # Parameters
    ///
    /// * `timeslots`: Vector of TimeSlots for the schedule
    ///
    /// # Returns
    ///
    /// A new `Schedule` instance with the provided parameters.
    pub fn new(id: Option<i32>, num_of_timeslots: i32, timeslots: Vec<TimeSlot>) -> Self {
        Self {
            id,
            num_of_timeslots,
            timeslots,
        }
    }
}

impl IntoResponse for &Schedule {
    /// Converts a `&Schedule` into an HTTP response.
    ///
    /// # Returns
    ///
    /// A `Response` object with a status code of 200 OK and a JSON body containing the schedule data.
    fn into_response(self) -> Response {
        tracing::info!("{:?}", &self);
        (StatusCode::OK, Json(&self)).into_response()
    }
}


/// Retrieves a paginated list of schedules from the schedule .
///
/// # Parameters
///
/// * `page`: The page number to retrieve (starts at 1)
/// * `limit`: The number of schedules to retrieve per page.
///
/// # Returns
///
/// A vector of Schedule's
/// If the pagination parameters are invalid, returns a `ScheduleErr` error.
pub async fn schedules_get(
    db_pool: &Pool<Postgres>,
) -> Result<Option<Schedule>, Box<dyn Error>> {
    let mut schedules: Vec<Schedule> = sqlx::query_as::<Postgres, Schedule>(
        r#"
        SELECT * FROM schedules
        ORDER BY id"#
    )
        .fetch_all(db_pool)
        .await?;
    trace!("schedules get vec: {:?}", &schedules);

    for schedule in &mut schedules {
        let timeslots = sqlx::query_as::<Postgres, TimeSlot>(
            "SELECT * FROM time_slots WHERE schedule_id = $1 ORDER BY start_time ASC;",
        )
        .bind(schedule.id)
        .fetch_all(db_pool)
        .await?;

        tracing::trace!("timeslots from schedule get: \n{:?}", &timeslots);
        schedule.timeslots = timeslots;
    }

    Ok(schedules.first().cloned())
}

/// Retrieves a schedule by its ID.
///
/// # Parameters
///
/// * `index`: The ID of the schedule.
///
/// # Returns
///
/// A reference to the `Schedule` instance with the specified ID, or a `ScheduleErr` error if the schedule does not exist.
pub async fn schedule_get(schedules: &Pool<Postgres>, index: i32) -> Result<Schedule, Box<dyn Error>> {
    let schedule_vec = sqlx::query_as::<Postgres, Schedule>(
        r#"select ts.*, t.*, sched.*, s.* from time_slots ts
        join schedules sched on ts.schedule_id = sched.id
        left join topics t on t.id = ts.topic_id
        left join speakers s on ts.speaker_id = s.id
        where ts.schedule_id = $1
        group by ts.id, t.id, s.id, sched.id;"#
    )
    .bind(index)
    .fetch_one(schedules)
    .await?;

    Ok(schedule_vec)
}

/// Adds a new schedule.
///
/// # Parameters
///
/// * `schedule`: The `Schedule` to add to the schedule .
///
/// # Returns
///
/// A `Result` indicating whether the schedule was added successfully.
/// If the schedule already exists, returns a `ScheduleErr` error.
pub async fn schedule_add(
    schedules: &Pool<Postgres>,
    Json(schedule_form): Json<CreateScheduleForm>
) -> Result<Schedule, Box<dyn Error>> {
    let schedule_row: (i32,) = sqlx::query_as(r#"INSERT INTO schedules (num_of_timeslots) VALUES ($1) RETURNING id"#)
        .bind(schedule_form.num_of_timeslots)
        .fetch_one(schedules)
        .await?;

    let schedule_id = schedule_row.0;
    let mut timeslots = vec![];
    for i in 0..(schedule_form.num_of_timeslots as usize) {
        let start_time = NaiveTime::parse_from_str(&schedule_form.start_time[i], "%H:%M")
            .map_err(|_| ScheduleErr::InvalidTimeFormat("Invalid time format. Expected format of \"%H:M\"".to_string()))?;
        let end_time = NaiveTime::parse_from_str(&schedule_form.end_time[i], "%H:%M")
            .map_err(|_| ScheduleErr::InvalidTimeFormat("Invalid time format. Expected format of \"%H:M\"".to_string()))?;

        let mut timeslot = TimeSlot::new(
            None,
            start_time,
            end_time,
            None,
            Some(schedule_id),
            None
        );
        let timeslot_id = timeslot_add(schedules, timeslot.clone()).await?;
        timeslot.id = Some(timeslot_id);
        timeslots.push(timeslot);
    }

    Ok(
        Schedule::new(Some(schedule_id), schedule_form.num_of_timeslots, timeslots)
    )
}

/// Removes a schedule by its ID.
///
/// # Parameters
///
/// * `index`: The ID of the schedule.
///
/// # Returns
///
/// A `Result` indicating whether the schedule was removed successfully.
/// If the schedule does not exist, returns a `ScheduleErr` error.
pub async fn schedule_delete(schedules: &Pool<Postgres>, index: i32) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        DELETE FROM schedules
        WHERE id = $1
        "#,
    )
    .bind(index)
    .execute(schedules)
    .await?;

    Ok(())
}

pub async fn schedule_update(schedules: &Pool<Postgres>, index: i32, schedule: Schedule) -> Result<Schedule, Box<dyn Error>> {
    let mut schedule_to_update = sqlx::query_as::<Postgres, Schedule>(
        r#"SELECT * FROM schedules WHERE id = $1"#,
    )
        .bind(index)
        .fetch_one(schedules)
        .await?;

    if schedule.num_of_timeslots != schedule_to_update.num_of_timeslots {
        schedule_to_update.num_of_timeslots = schedule.num_of_timeslots;
        sqlx::query(
            r#"
            UPDATE FROM schedules (num_of_timeslots) VALUES ($1)
            WHERE id = $2
            "#,
        )
            .bind(schedule.num_of_timeslots)
            .bind(index)
            .execute(schedules)
            .await?;
    }

    for timeslot in schedule.timeslots {
        if let Some(timeslot_to_update) = schedule_to_update.timeslots.iter_mut().find(|t| t.id == timeslot.id) {
            *timeslot_to_update = timeslot.clone();
            timeslot_update(schedules, &timeslot_to_update).await?;
        }
    }

    Ok(schedule_to_update)
}


pub async fn schedule_generate(db_pool: &Pool<Postgres>) -> Result<Schedule, Box<dyn Error>> {
    let topics = get_all_topics(db_pool).await?;
    let num_of_topics = topics.len();
    let mut timeslots = vec![];
    let mut schedule = schedules_get(db_pool).await?.ok_or("Error getting schedule")?;
    let schedule_id = schedule.id.ok_or("Error getting schedule ID")?;

    for i in 0..(schedule.num_of_timeslots as usize) {
        if i < num_of_topics {
            let topic = &topics[i];
            trace!("timeslots: {:?}", &schedule.timeslots);
            let timeslot = &schedule.timeslots[i];
            let updated_timeslot = TimeSlot::new(
                timeslot.id,
                timeslot.start_time,
                timeslot.end_time,
                Some(topic.speaker_id),
                Some(schedule_id),
                topic.id
            );

            timeslot_update(db_pool, &updated_timeslot).await?;
            timeslots.push(updated_timeslot);

        } else {
            break;
        }
    }

    schedule = schedule_update(db_pool, schedule_id, schedule.clone()).await?;

    tracing::trace!("schedule generate sched: {:?}", &schedule);
    Ok(schedule)
}
