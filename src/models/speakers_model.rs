use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Response, Json};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use std::error::Error;
use utoipa::{
    ToSchema,
};
use crate::types::ApiStatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
/// A struct representing a speaker
/// 
/// This struct represents a speaker with a name, email, and phone number.
/// 
/// # Fields
/// - `speaker_id` - The ID of the speaker
/// - `name` - The name of the speaker
/// - `email` - The email of the speaker
/// - `phone_number` - The phone number of the speaker
pub struct Speaker {
    pub speaker_id: Option<i32>,
    pub name: String,
    pub email: String,
    pub phone_number: String,
}

/// An enumeration of possible errors that can occur when working with speakers.
/// 
/// This enum represents the possible errors that can occur when working with speakers.
/// 
/// # Variants
/// - `DoesNotExist` - The speaker does not exist
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum SpeakerErr {
    #[error("Speaker {0} doesn't exist")]
    DoesNotExist(String),
}

/// Struct representing an error that occurred when working with speakers.
/// 
/// This struct represents an error that occurred when working with speakers.
/// 
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
#[derive(Debug, ToSchema)]
pub struct SpeakerError {
    pub status: ApiStatusCode,
    pub error: String,
}

/// Implements `ToSchema` trait for `SpeakerError` struct.
/// 
/// This implementation provides a JSON schema for the `SpeakerError` struct. The schema defines two
/// properties: `status` and `error`.

/// Implements the `Serialize` trait for `SpeakerError`
/// 
/// This implementation serializes a `SpeakerError` into a JSON object with two properties: `status`
/// and `error`.
impl Serialize for SpeakerError {
    /// Serializes a `SpeakerError`
    ///
    /// The serialized JSON object will have two properties:
    /// - `status`: A string for the HTTP status code
    /// - `error`: A string describing the error
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status: String = self.status.to_string();
        let mut state = serializer.serialize_struct("SpeakerError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl SpeakerError {
    /// Creates a `Response` instance from a `StatusCode` and `SpeakerErr`.
    ///
    /// This function creates a `Response` instance from a `StatusCode` and a `SpeakerErr`. The
    /// `SpeakerErr` is serialized into a JSON object with two properties: `status` and `error`.
    /// 
    /// # Parameters
    /// - `status`: The HTTP status code to return
    /// - `error`: The `SpeakerErr` error to return
    /// 
    /// # Returns
    /// A `Response` instance with the provided status code and error.
    pub fn response(status: ApiStatusCode, error: Box<dyn Error>) -> Response {
        let error = SpeakerError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
}

impl Speaker {
    /// Creates a new `Speaker` instance.
    ///
    /// This function creates a new `Speaker` instance with the provided speaker ID, name, email, 
    /// and phone number.
    /// 
    /// # Parameters
    /// - `Option<speaker_id>`: The ID of the speaker or None if the speaker is new
    /// - `name`: The name of the speaker
    /// - `email`: The email of the speaker
    /// - `phone_number`: The phone number of the speaker
    /// 
    /// # Returns
    /// A new `Speaker` instance
    pub fn new(speaker_id: Option<i32>, name: String, email: String, phone_number: String) -> Self {
        Self {
            speaker_id,
            name,
            email,
            phone_number,
        }
    }
}

/// Implements the `IntoResponse` trait for `&Speaker` struct.
/// 
/// This implementation converts a `&Speaker` into an HTTP response. The response has a status code
/// of 200 OK and a JSON body containing the speaker data.
impl IntoResponse for &Speaker {
    /// Converts a `&Speaker` into an HTTP response.
    ///
    /// This function converts a `&Speaker` into an HTTP response with a status code of 200 OK and a
    /// JSON body containing the speaker data.
    /// 
    /// # Returns
    /// An HTTP response with a status code of 200 OK and a JSON body containing the speaker data.
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self)).into_response()
    }
}

/// Retrieves a list of speakers from the database.
/// 
/// This function retrieves a list of speakers from the database.
/// 
/// # Parameters
/// - `db_pool`: The database connection pool
/// 
/// # Returns
/// A vector of `Speaker` instances representing the speakers in the database or an error if the
/// query fails.
/// 
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn speakers_get(
    db_pool: &Pool<Postgres>,
) -> Result<Vec<Speaker>, Box<dyn Error>> {
    let speakers: Vec<Speaker> = sqlx::query_as(
        r#"
        SELECT * FROM speakers;"#,
    )
    .fetch_all(db_pool)
    .await?;

    Ok(speakers)
}

/// Retrieves a speaker by its ID.
///
/// This function retrieves a speaker by its ID.
/// 
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `index`: The ID of the speaker to retrieve
/// 
/// # Returns
/// A `Speaker` instance representing the speaker with the provided ID or an error if the query
/// fails.
/// 
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn speaker_get(
    db_pool: &Pool<Postgres>,
    index: i32,
) -> Result<Speaker, Box<dyn Error>> {
    let speaker = sqlx::query_as::<Postgres, Speaker>("SELECT id as speaker_id, name, email, \
    phone_number FROM speakers where id = $1")
        .bind(index)
        .fetch_one(db_pool)
        .await?;

    Ok(speaker)
}

/// Adds a new speaker.
///
/// This function adds a new speaker to the database.
/// 
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `speaker`: The `Speaker` instance to add
/// 
/// # Returns
/// The ID of the newly added speaker or an error if the query fails.
/// 
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn speaker_add(
    db_pool: &Pool<Postgres>,
    speaker: Speaker,
) -> Result<i32, Box<dyn Error>> {
    let (speaker_id,) = sqlx::query_as(
        "INSERT INTO speakers (name, email, phone_number) VALUES ($1, $2, $3) RETURNING id",
    )
        .bind(speaker.name)
        .bind(speaker.email)
        .bind(speaker.phone_number)
        .fetch_one(db_pool)
        .await?;

    Ok(speaker_id)
}

/// Removes a speaker by its ID.
///
/// This function removes a speaker by its ID.
/// 
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `index`: The ID of the speaker to remove
/// 
/// # Returns
/// An empty `Result` if the speaker was removed successfully or an error if the query fails.
/// 
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn speaker_delete(db_pool: &Pool<Postgres>, index: i32) -> Result<(), Box<dyn Error>> {
    sqlx::query_as::<Postgres, Speaker>(
        "DELETE FROM speakers
        WHERE id = $1;",
    )
        .bind(index)
        .fetch_all(db_pool)
        .await?;

    Ok(())
}

/// Updates a speaker by its ID.
///
/// This function updates a speaker by its ID.
/// 
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `index`: The ID of the speaker to update
/// - `speaker`: The `Speaker` instance with the data to use for the update
/// 
/// # Returns
/// The updated `Speaker` instance or an error if the query fails.
/// 
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn speaker_update(
    db_pool: &Pool<Postgres>,
    index: i32,
    speaker: Speaker,
) -> Result<Speaker, Box<dyn Error>> {
    let name = speaker.name;
    let email = speaker.email;
    let phone_number = speaker.phone_number;

    let mut speaker_to_update = speaker_get(db_pool, index).await?;
    tracing::debug!("speaker to update: {:?}", &speaker_to_update);
    speaker_to_update.name.clone_from(&name);
    speaker_to_update.email.clone_from(&email);
    speaker_to_update.phone_number.clone_from(&phone_number);

    sqlx::query_as::<Postgres, Speaker>(
        "UPDATE speakers
        SET name = $1, email = $2, phone_number = $3
        WHERE id = $4;",
    )
        .bind(name)
        .bind(email)
        .bind(phone_number)
        .bind(index)
        .fetch_all(db_pool)
        .await?;
    
    Ok(speaker_to_update)
}