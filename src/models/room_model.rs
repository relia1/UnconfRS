use crate::types::ApiStatusCode;
use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Response, Json};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use std::error::Error;
use utoipa::ToSchema;

/// A boxed error type for use in functions that return a `Result` with an error type of
/// `BoxedError`.
///
/// This type is a `Box` containing a trait object that implements the `Error`, `Send`, and `Sync`
/// traits.
type BoxedError = Box<dyn Error + Send + Sync>;

/// Enum representing the possible errors that can occur when working with rooms.
///
/// This enum implements the `Error`, `ToSchema`, and `Serialize` traits and has two variants.
///
/// # Variants:
/// - `IoError(String)`: An I/O error occurred.
/// - `DoesNotExist(String)`: The room does not exist.
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

/// Struct representing a room error.
///
/// Fields:
/// - `status`: The HTTP status code associated with the error.
/// - `error`: A string describing the specific error that occurred.
#[derive(Debug, ToSchema)]
pub struct RoomError {
    pub status: ApiStatusCode,
    pub error: String,
}

/// Implements the `Serialize` trait for `RoomError`.
///
/// This trait implementation allows a `RoomError` to be serialized to JSON. The serialized JSON
/// object will have two properties:
/// - `status`: A string for the HTTP status code.
/// - `error`: A string describing the error.
impl Serialize for RoomError {
    /// Serializes a `RoomError`
    ///
    /// The serialized JSON object will have two properties:
    ///
    /// - `status`: A string for the HTTP status code
    /// - `error`: A string describing the error
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
    /// Creates a new `RoomError` instance.
    ///
    /// This function takes an HTTP status code and an error message and returns a new `RoomError`
    /// instance.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code associated with the error.
    /// - `error`: A string describing the specific error that occurred.
    ///
    /// # Returns
    /// A new `RoomError` instance with the provided status code and error message.
    pub fn response(status: ApiStatusCode, error: BoxedError) -> Response {
        let error = RoomError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
/// Struct representing a room.
///
/// This struct represents a room with an ID, available spots, name, and location.
///
/// Fields:
/// - `id`: The unique identifier for the room.
/// - `available_spots`: The number of available timeslots for the room.
/// - `name`: The name of the room.
/// - `location`: The location of the room.
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
    /// This function takes an optional ID, the number of available spots, the name, and the
    /// location of the room and returns a new `Room` instance.
    ///
    /// # Parameters
    /// - `id`: An optional ID for the room.
    /// - `available_spots`: The number of available spots for the room.
    /// - `name`: The name of the room.
    /// - `location`: The location of the room.
    ///
    /// # Returns
    /// A new `Room` instance with the provided ID, available spots, name, and location.
    pub fn new(id: Option<i32>, available_spots: i32, name: String, location: String) -> Self {
        Self {
            id,
            available_spots,
            name,
            location,
        }
    }
}

/// Implements the `IntoResponse` trait for `Room`.
///
/// This trait implementation allows a `Room` instance to be converted into an HTTP response.
impl IntoResponse for &Room {
    /// Converts a `Room` instance into an HTTP response.
    ///
    /// This function converts a `Room` instance into an HTTP response with a status code of 200 OK.
    ///
    /// # Returns
    /// An HTTP response with a status code of 200 OK and the `Room` instance as JSON.
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self)).into_response()
    }
}
#[derive(Debug, Deserialize)]
/// Struct representing a form for creating rooms.
///
/// This struct represents a form for creating rooms. It contains a vector of `Room`
/// instances.
///
/// Fields:
/// - `rooms`: A vector of `Room` instances.
pub(crate) struct CreateRoomsForm {
    pub(crate) rooms: Vec<Room>,
}

/// Gets all rooms.
///
/// This function retrieves all rooms from the database.
///
/// # Parameters
/// - `db_pool`: A reference to the database connection pool.
///
/// # Returns
/// A `Result` containing an optional vector of `Room` instances. If the rooms are successfully
/// retrieved, the vector is returned. If no rooms are found, `None` is returned.
///
/// # Errors
/// If an error occurs while fetching the rooms from the database, a `BoxedError` is returned.
pub(crate) async fn rooms_get(db_pool: &Pool<Postgres>) -> Result<Option<Vec<Room>>, BoxedError> {
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
/// This function adds a new room to the database.
///
/// # Parameters
/// - `db_pool`: A reference to the database connection pool.
/// - `rooms_form`: The form containing the room to add
///
/// # Returns
/// A `Result` containing a `Schedule` instance. If the room is successfully added, the schedule
/// containing the room is returned otherwise an error is returned.
///
/// # Errors
/// If an error occurs while adding the room to the database, a `BoxedError` is returned.
pub async fn rooms_add(
    db_pool: &Pool<Postgres>,
    rooms_form: CreateRoomsForm,
) -> Result<(), BoxedError> {
    let tx = db_pool.begin().await?;
    for room in &rooms_form.rooms {
        sqlx::query_as::<Postgres, Room>(
            r#"INSERT INTO rooms (name,
        available_spots, 
        location) 
        VALUES 
        ($1, $2, $3) RETURNING id, available_spots, name, location"#,
        )
            .bind(room.name.clone())
            .bind(room.available_spots)
            .bind(room.location.clone())
            .fetch_one(db_pool)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Removes a room by ID.
///
/// This function removes a room from the database by its ID.
///
/// # Parameters
/// - `db_pool`: A reference to the database connection pool.
/// - `index`: The ID of the room to remove.
///
/// # Returns
/// A `Result` - if the room is successfully removed, `Ok(())` is returned. If the room does not
/// exist, an error is returned.
///
/// # Errors
/// If an error occurs while removing the room from the database, a `BoxedError` is returned.
pub async fn room_delete(db_pool: &Pool<Postgres>, index: i32) -> Result<(), BoxedError> {
    sqlx::query(
        r#"
        DELETE FROM timeslot_assignments
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
