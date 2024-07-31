use std::error::Error;


use askama_axum::IntoResponse;
use axum::{http::StatusCode, Json, response::Response};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres, Row};
use utoipa::{openapi::{ObjectBuilder, RefOr, Schema, SchemaType}, ToSchema};


/// An enumeration of errors that may occur
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum TimeSlotErr {
    #[error("TimeSlot io failed: {0}")]
    IoError(String),
    #[error("TimeSlot {0} doesn't exist")]
    DoesNotExist(String),
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
        schemas(TopicWithoutId, TopicError)
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TimeSlotWithoutId {
    pub start_time: i64, // unix timestamp (seconds since epoch)
    pub end_time: i64, // unix timestamp (seconds since epoch)
    pub duration: i64, // duration in seconds
}

impl TimeSlotWithoutId {
    pub fn new(
        start_time: i64,
        end_time: i64,
        duration: i64,
    )
    -> Self {
        Self {
            start_time,
            end_time,
            duration,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TimeSlot {
    pub id: i32,
    pub start_time: i64, // unix timestamp (seconds since epoch)
    pub end_time: i64, // unix timestamp (seconds since epoch)
    pub duration: i64, // duration in seconds
    // pub timeslot: Option<TimeSlot>,
    // pub timeslots_id: Option<i32>,
    // pub timeslot_id: Option<i32>,
}

impl TimeSlot {
    pub fn new(
        id: i32,
        start_time: i64,
        end_time: i64,
        duration: i64,
        // timeslots_id: Option<i32>,
        // timeslot_id: Option<i32>
    )
    -> Self {
        Self {
            id,
            start_time,
            end_time,
            duration,
            //timeslots_id,
            //timeslot_id,
        }
    }
}

/// Retrieves a paginated list of timeslots from the timeslot .
///
/// # Parameters
///
/// * `page`: The page number to retrieve (starts at 1)
/// * `limit`: The number of timeslots to retrieve per page.
///
/// # Returns
///
/// A vector of TimeSlot's
/// If the pagination parameters are invalid, returns a `TimeSlotErr` error.
pub async fn timeslot_paginated_get(
    timeslots: &Pool<Postgres>,
    page: i32,
    limit: i32,
) -> Result<Vec<TimeSlot>, Box<dyn Error>> {
    if page < 1 || limit < 1 {
        return Err(Box::new(TimeSlotErr::PaginationInvalid(
            "Page and limit must be positive".to_string(),
        )));
    }

    let num_timeslots: i64 = sqlx::query("SELECT COUNT(*) FROM timeslots")
        .fetch_one(timeslots)
        .await?
        .get(0);

    let start_index = ((page - 1) * limit) as i64;
    if start_index > num_timeslots {
        return Err(Box::new(TimeSlotErr::PaginationInvalid(
            "Invalid query parameter values".to_string(),
        )));
    }

    let timeslots: Vec<TimeSlot> = sqlx::query_as(
        r#"
        SELECT * FROM timeslots
        ORDER BY id
        LIMIT $1 OFFSET $2;"#,
    )
        .bind(limit)
        .bind(start_index)
        .fetch_all(timeslots)
        .await?;

    Ok(timeslots)
/*





    let row = sqlx::query_as::<Postgres, TimeSlot>(
        "SELECT * FROM time_slots;"
    )
    .fetch_one(timeslots)
    .await?;

    let time_slots = sqlx::query_as::<Postgres, TimeSlot>(
        "SELECT * FROM time_slots"
    )
    .fetch_all(timeslots)
    .await?;

    let mut timeslot_vec: Vec<TimeSlot> = Vec::new();
    /*
    for row in timeslots {
        timeslot_vec.push(row.into() );
    }*/

    Ok(timeslot_vec)*/
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
pub async fn timeslot_get(timeslots: &Pool<Postgres>, index: i32) -> Result<Vec<TimeSlot>, Box<dyn Error>> {
    let timeslots: Vec<_> = sqlx::query_as::<Postgres, TimeSlot>(
        "SELECT *
        FROM time_slots
        WHERE id = $1;",
    )
    .bind(index)
    .fetch_all(timeslots)
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
pub async fn timeslot_add(timeslots: &Pool<Postgres>,) -> Result<(), Box<dyn Error>> {
    sqlx::query(r#"INSERT INTO timeslots DEFAULT VALUES RETURNING id"#)
        .fetch_one(timeslots)
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
pub async fn timeslot_delete(timeslots: &Pool<Postgres>, index: i32) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        DELETE FROM timeslots
        WHERE id = $1
        "#,
    )
    .bind(index)
    .execute(timeslots)
    .await?;

    Ok(())
}
