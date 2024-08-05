use std::error::Error;


use askama_axum::IntoResponse;
use axum::{http::StatusCode, Json, response::Response};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{Pool, Postgres, FromRow};
use utoipa::{openapi::{ObjectBuilder, RefOr, Schema, SchemaType}, ToSchema};

use crate::{timeslot_model::*, topics_model::get_all_topics};


/// An enumeration of errors that may occur
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum ScheduleErr {
    #[error("Schedule io failed: {0}")]
    IoError(String),
    #[error("Schedule {0} doesn't exist")]
    DoesNotExist(String),
    #[error("Invalid query parameter values")]
    PaginationInvalid(String),
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
pub struct ScheduleWithoutId {
    pub num_of_timeslots: i32,
    //#[sqlx(skip)]
    //pub timeslots: Vec<TimeSlotWithoutId>,
}

impl ScheduleWithoutId {
    /// Creates a new `Schedule` instance.
    ///
    /// # Parameters
    ///
    /// * `timeslots`: Vector of TimeSlots for the schedule
    ///
    /// # Returns
    ///
    /// A new `Schedule` instance with the provided parameters.
    pub fn new(num_of_timeslots: i32/*, timeslots: Vec<TimeSlotWithoutId>*/) -> Self {
        Self {
            num_of_timeslots,
            //timeslots,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct Schedule {
    pub id: i32,
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
    pub fn new(id: i32, num_of_timeslots: i32, timeslots: Vec<TimeSlot>) -> Self {
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

    for schedule in &mut schedules {
        let timeslots = sqlx::query_as::<_, TimeSlot>(
            "SELECT * FROM time_slots WHERE schedule_id = $1 ORDER BY start_time;",
        )
        .bind(schedule.id)
        .fetch_all(db_pool)
        .await?;

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
    /*
    let schedule_vec = vec![];
    let schedule_info = sqlx::query_as<Postgres, Schedule>(
        "SELECT * FROM schedules WHERE id = $1;"
    )
        .bind(index)
        .fetch_one(schedules)
        .await?;

    let time_slots = sqlx::query_as::<Postgres, TimeSlot>(
        "SELECT ts.*, t.title, t.content, s.name as speaker_name, s.email, s.phone_number
        FROM time_slots ts
        JOIN topics t ON ts.topic_id = t.id
        JOIN speakers s ON ts.speaker_id = s.id
        WHERE ts.schedule_id = $1;",
    )
        .bind(index)
        .fetch_all(schedules)
        .await?;

    let schedule = Schedule {
        id: index,
        num_of_timeslots: schedule_info.num_of_timeslots,
        timeslots: time_slots.into_iter().map(|ts| {
            id: ts.id,
        })
    }
*/
    let schedule_vec = sqlx::query_as::<Postgres, Schedule>(
        "select ts.*, t.*, sched.*, s.* from time_slots ts
join schedules sched on ts.schedule_id = sched.id
left join topics t on t.id = ts.topic_id
left join speakers s on ts.speaker_id = s.id
where ts.schedule_id = 1
group by ts.id, t.id, s.id, sched.id;"/*
        "SELECT ts.id, ts.start_time, ts.end_time, ts.duration, ts.speaker_id,
             t.id as topic_id, t.title, t.content,
             s.id as speaker_id, s.name, s.email, s.phone_number,
             sched.num_of_timeslots
        FROM time_slots ts
        JOIN schedules sched ON ts.speaker_id = sched.id
        JOIN topics t ON ts.topic_id = t.id
        JOIN speakers s ON ts.speaker_id = s.id
        WHERE ts.schedule_id = $1
        GROUP BY ts.id, t.id, s.id, sched.num_of_timeslots;",*/
    )
    .bind(index)
    .fetch_one(schedules)
    .await?;



/*
    let schedule_vec = vec![];
    let schedule = sqlx::query_as::<Postgres, TimeSlot>(
        "SELECT *
        FROM time_slots
        WHERE id = $1;",
    )
        .bind(index)
        .fetch_all(schedules)
        .await?;

    //schedule_vec.push(<Schedule as std::convert::From<PgRow>>::from(schedule));*/
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
pub async fn schedule_add(schedules: &Pool<Postgres>, schedule: ScheduleWithoutId) -> Result<i32, Box<dyn Error>> {
    let sched_row: (i32,) = sqlx::query_as(r#"INSERT INTO schedules (num_of_timeslots) VALUES ($1) RETURNING id"#)
        .bind(schedule.num_of_timeslots)
        .fetch_one(schedules)
        .await?;

    Ok(sched_row.0)
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

pub async fn schedule_update(schedules: &Pool<Postgres>, index: i32, schedule: Schedule) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        UPDATE FROM schedules
        WHERE id = $1
        "#,
    )
    .bind(index)
    .execute(schedules)
    .await?;

    Ok(())
}


pub async fn schedule_generate(db_pool: &Pool<Postgres>, num_of_timeslots: i32) -> Result<Schedule, Box<dyn Error>> {
    let topics = get_all_topics(db_pool).await?;
    let num_of_topics = topics.len();
    let schedule = ScheduleWithoutId::new(num_of_timeslots);
    let sched_id = schedule_add(db_pool, schedule.clone()).await?;
    let mut timeslots = vec![];
    tracing::debug!("num timeslots: {}", num_of_timeslots);

    for i in 0..(num_of_timeslots as usize) {
        if i < num_of_topics {
            let topic = &topics[i];
            let new_timeslot = TimeSlotWithoutId::new(0, 0, 0, topic.speaker_id, sched_id, Some(topic.id));
            let timeslot_id = timeslot_add(db_pool, new_timeslot.clone()).await?;
            timeslots.push(TimeSlot::new(timeslot_id, 0, 0, 0, topic.speaker_id, sched_id, Some(topic.id)));
        } else {
            let new_timeslot = TimeSlotWithoutId::new(-1, -1, -1, -1, sched_id, None);
            let timeslot_id = timeslot_add(db_pool, new_timeslot.clone()).await?;
            timeslots.push(TimeSlot::new(timeslot_id, -1, -1, -1, -1, sched_id, None));
        }
    }

    let sched = Schedule::new(sched_id, num_of_timeslots, timeslots);
    tracing::trace!("schedule generate sched: {:?}", &sched);
    Ok(sched)
}
