use std::error::Error;

use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Response, Json};
use chrono::NaiveTime;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use utoipa::{
    openapi::{ObjectBuilder, RefOr, Schema, SchemaType},
    ToSchema,
};

use crate::models::timeslot_model::{timeslot_add, TimeSlot};
use crate::{models::schedule_model::*, CreateRoomsForm};

/// An enumeration of errors that may occur
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum RoomErr {
    #[error("Room io failed: {0}")]
    IoError(String),
    #[error("Room {0} doesn't exist")]
    DoesNotExist(String),
}

impl From<std::io::Error> for RoomErr {
    /// Converts a `std::io::Error` into a `RoomErr`.
    ///
    /// # Description
    ///
    /// This allows `std::io::Error` instances to be converted into
    /// `RoomErr`, wrapping the I/O error as a `RoomIoError`.
    ///
    /// # Example
    ///
    /// ```
    /// let io_err = std::io::Error::new(std::io::ErrorKind::Other, "IO error");
    /// let room_err: RoomErr = io_err.into();
    /// ```
    fn from(e: std::io::Error) -> Self {
        RoomErr::IoError(e.to_string())
    }
}

/// struct that represents a Room error, but include a `StatusCode`
/// in addition to a `RoomErr`
#[derive(Debug)]
pub struct RoomError {
    pub status: StatusCode,
    pub error: String,
}

/// Implements `ToSchema` trait for `RoomError` generating a JSON schema
/// for the error type
impl<'s> ToSchema<'s> for RoomError {
    /// Returns a JSON schema for `RoomError`
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
                "status":"404","error":"no room"
            })))
            .into();
        ("RoomError", sch)
    }
}

/// Implements the `Serialize` trait for `RoomError`
impl Serialize for RoomError {
    /// Serializes a `RoomError`
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
        let mut state = serializer.serialize_struct("RoomError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl RoomError {
    /// Creates a `Response` instance from a `StatusCode` and `RoomErr`.
    ///
    /// # Parameters
    ///
    /// * `status`: The HTTP status code.
    /// * `error`: The `RoomErr` instance.
    ///
    /// # Returns
    ///
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: StatusCode, error: Box<dyn Error>) -> Response {
        let error = RoomError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct Room {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub available_spots: i32,
    pub name: String,
    pub location: String,
}

impl Room {
    /// Creates a new `Room` instance.
    ///
    /// # Parameters
    ///
    /// * `Name`: Name of the room
    /// * `Location`: Location of the room
    ///
    /// # Returns
    ///
    /// A new `Room` instance with the provided parameters.
    pub fn new(id: Option<i32>, available_spots: i32, name: String, location: String) -> Self {
        Self {
            id,
            available_spots,
            name,
            location,
        }
    }
}

impl IntoResponse for &Room {
    /// Converts a `&Room` into an HTTP response.
    ///
    /// # Returns
    ///
    /// A `Response` object with a status code of 200 OK and a JSON body containing the room data.
    fn into_response(self) -> Response {
        tracing::info!("{:?}", &self);
        (StatusCode::OK, Json(&self)).into_response()
    }
}

/// Retrieves a list of rooms
///
/// # Returns
///
/// A vector of Room's or None
pub(crate) async fn rooms_get(
    db_pool: &Pool<Postgres>,
) -> Result<Option<Vec<Room>>, Box<dyn Error>> {
    let rooms = Some(
        sqlx::query_as::<Postgres, Room>(
            r#"
        SELECT * FROM rooms
        ORDER BY id"#,
        )
        .fetch_all(db_pool)
        .await?,
    );

    Ok(rooms.filter(|res| !res.is_empty()))
}

/// Adds a new room.
///
/// # Parameters
///
/// * `room`: The `Room` to add
///
/// # Returns
///
/// A `Result` indicating whether the room was added successfully.
pub async fn rooms_add(
    db_pool: &Pool<Postgres>,
    Json(rooms_form): Json<CreateRoomsForm>,
) -> Result<Schedule, Box<dyn Error>> {
    for room in rooms_form.rooms {
        sqlx::query_as(r#"INSERT INTO rooms (name, available_spots, location) VALUES ($1, $2, $3) RETURNING id"#)
            .bind(room.name)
            .bind(room.available_spots)
            .bind(room.location)
            .fetch_one(db_pool)
            .await?;
    }

    let (schedule_id,) =
        sqlx::query_as(r#"INSERT INTO schedules (num_of_timeslots) VALUES ($1) RETURNING id"#)
            .bind(20)
            .fetch_one(db_pool)
            .await?;

    let mut timeslots = vec![];

    let rooms = rooms_get(db_pool).await?.unwrap();
    for room in rooms {
        for i in 8..18 {
            let start_time = NaiveTime::parse_from_str(&format!("{}:{}", i, 0), "%H:%M").unwrap();
            let end_time = NaiveTime::parse_from_str(&format!("{}:{}", i, 30), "%H:%M").unwrap();

            let start_time2 = NaiveTime::parse_from_str(&format!("{}:{}", i, 30), "%H:%M").unwrap();
            let end_time2 =
                NaiveTime::parse_from_str(&format!("{}:{}", i + 1, 00), "%H:%M").unwrap();

            let mut timeslot = TimeSlot::new(
                None,
                start_time,
                end_time,
                None,
                Some(schedule_id),
                None,
                room.id,
            );

            let mut timeslot2 = TimeSlot::new(
                None,
                start_time2,
                end_time2,
                None,
                Some(schedule_id),
                None,
                room.id,
            );

            let timeslot_id = timeslot_add(db_pool, timeslot.clone()).await?;
            let timeslot_id2 = timeslot_add(db_pool, timeslot2.clone()).await?;
            timeslot.id = Some(timeslot_id);
            timeslot2.id = Some(timeslot_id2);
            timeslots.push(timeslot);
            timeslots.push(timeslot2);
        }
    }

    Ok(Schedule::new(Some(schedule_id), 20, timeslots))
}

/// Removes a room by its ID.
///
/// # Parameters
///
/// * `index`: The ID of the room.
///
/// # Returns
///
/// A `Result` indicating whether the room was removed successfully.
/// If the room does not exist, returns a `RoomErr` error.
pub async fn room_delete(db_pool: &Pool<Postgres>, index: i32) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        DELETE FROM time_slots
        WHERE room_id = $1
        "#,
    )
    .bind(index)
    .execute(db_pool)
    .await?;
    
    sqlx::query(
        r#"
        DELETE FROM rooms
        WHERE id = $1
        "#,
    )
    .bind(index)
    .execute(db_pool)
    .await?;

    Ok(())
}
