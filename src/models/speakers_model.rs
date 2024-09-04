use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Response, Json};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres, Row};
use std::error::Error;
use utoipa::{
    openapi::{ObjectBuilder, RefOr, Schema, SchemaType},
    ToSchema,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct Speaker {
    pub speaker_id: Option<i32>,
    pub name: String,
    pub email: String,
    pub phone_number: String,
}

/// An enumeration of errors that may occur
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum SpeakerErr {
    #[error("Speaker {0} doesn't exist")]
    DoesNotExist(String),
    #[error("Invalid query parameter values")]
    PaginationInvalid(String),
}

/// struct that represents a Speaker error, but include a `StatusCode`
/// in addition to a `SpeakerErr`
#[derive(Debug)]
pub struct SpeakerError {
    pub status: StatusCode,
    pub error: String,
}

/// Implements `ToSchema` trait for `SpeakerError` generating a JSON schema
/// for the error type
impl<'s> ToSchema<'s> for SpeakerError {
    /// Returns a JSON schema for `SpeakerError`
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
                "status":"404","error":"no speaker"
            })))
            .into();
        ("SpeakerError", sch)
    }
}

/// Implements the `Serialize` trait for `SpeakerError`
impl Serialize for SpeakerError {
    /// Serializes a `SpeakerError`
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
        let mut state = serializer.serialize_struct("SpeakerError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl SpeakerError {
    /// Creates a `Response` instance from a `StatusCode` and `SpeakerErr`.
    ///
    /// # Parameters
    ///
    /// * `status`: The HTTP status code.
    /// * `error`: The `SpeakerErr` instance.
    ///
    /// # Returns
    ///
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: StatusCode, error: Box<dyn Error>) -> Response {
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
    /// # Parameters
    ///
    /// * `id`: ID for the speaker.
    /// * `Name`: The name of the speaker.
    /// * `Email`: The email of the speaker.
    /// * `Phone Number`: The phone number of the speaker.
    ///
    /// # Returns
    ///
    /// A new `Speaker` instance with the provided parameters.
    pub fn new(speaker_id: Option<i32>, name: String, email: String, phone_number: String) -> Self {
        Self {
            speaker_id,
            name,
            email,
            phone_number,
        }
    }
}

impl IntoResponse for &Speaker {
    /// Converts a `&Speaker` into an HTTP response.
    ///
    /// # Returns
    ///
    /// A `Response` object with a status code of 200 OK and a JSON body containing the speaker data.
    fn into_response(self) -> Response {
        tracing::info!("{:?}", &self);
        (StatusCode::OK, Json(&self)).into_response()
    }
}

/// Retrieves a paginated list of speakers from the speaker bank.
///
/// # Parameters
///
/// * `page`: The page number to retrieve (starts at 1)
/// * `limit`: The number of speakers to retrieve per page.
///
/// # Returns
///
/// A vector of Speaker's
/// If the pagination parameters are invalid, returns a `SpeakerErr` error.
pub async fn speaker_paginated_get(
    db_pool: &Pool<Postgres>,
    page: i32,
    limit: i32,
) -> Result<Vec<Speaker>, Box<dyn Error>> {
    if page < 1 || limit < 1 {
        return Err(Box::new(SpeakerErr::PaginationInvalid(
            "Page and limit must be positive".to_string(),
        )));
    }

    let num_speakers: i64 = sqlx::query(r#"SELECT COUNT(*) FROM speakers;"#)
        .fetch_one(db_pool)
        .await?
        .get(0);

    let start_index = (page - 1) * limit;
    if start_index as i64 > num_speakers {
        return Err(Box::new(SpeakerErr::PaginationInvalid(
            "Invalid query parameter values".to_string(),
        )));
    }

    let speakers: Vec<Speaker> = sqlx::query_as(
        r#"
        SELECT * FROM speakers
        LIMIT $1 OFFSET $2;"#,
    )
    .bind(limit)
    .bind(start_index)
    .fetch_all(db_pool)
    .await?;

    Ok(speakers)
}

/// Retrieves a speaker by its ID.
///
/// # Parameters
///
/// * `index`: The ID of the speaker.
///
/// # Returns
///
/// A reference to the `Speaker` instance with the specified ID, or a `SpeakerErr` error if the speaker does not exist.
pub async fn speaker_get(
    db_pool: &Pool<Postgres>,
    index: i32,
) -> Result<Vec<Speaker>, Box<dyn Error>> {
    let mut speaker_vec = vec![];
    let speaker = sqlx::query_as::<Postgres, Speaker>("SELECT * FROM speakers where id = $1")
        .bind(index)
        .fetch_one(db_pool)
        .await?;

    // speaker_vec.push(<Speaker as std::convert::From<PgRow>>::from(speaker));
    speaker_vec.push(speaker);
    Ok(speaker_vec)
}

/// Adds a new speaker.
///
/// # Parameters
///
/// * `speaker`: The `Speaker` to add to the speaker bank.
///
/// # Returns
///
/// A `Result` indicating whether the speaker was added successfully.
/// If the speaker already exists, returns a `SpeakerErr` error.
pub async fn speaker_add(
    db_pool: &Pool<Postgres>,
    speaker: Speaker,
) -> Result<i32, Box<dyn Error>> {
    tracing::debug!("adding speaker");
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO speakers (name, email, phone_number) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(speaker.name)
    .bind(speaker.email)
    .bind(speaker.phone_number)
    .fetch_one(db_pool)
    .await?;

    Ok(row.0)
}

/// Removes a speaker by its ID.
///
/// # Parameters
///
/// * `index`: The ID of the speaker.
///
/// # Returns
///
/// A `Result` indicating whether the speaker was removed successfully.
/// If the speaker does not exist, returns a `SpeakerErr` error.
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
/// # Parameters
///
/// * `index`: The ID of the speaker to update.
/// * `speaker`: The updated `Speaker` instance.
///
/// # Returns
///
/// A `Result` indicating whether the speaker was updated successfully.
/// If the speaker does not exist or is unprocessable, returns a `SpeakerErr` error.
/// If successful, returns a `StatusCode` of 200.
pub async fn speaker_update(
    db_pool: &Pool<Postgres>,
    index: i32,
    speaker: Speaker,
) -> Result<Vec<Speaker>, Box<dyn Error>> {
    let name = speaker.name;
    let email = speaker.email;
    let phone_number = speaker.phone_number;

    let mut speaker_to_update = speaker_get(db_pool, index).await?;
    speaker_to_update[0].name.clone_from(&name);
    speaker_to_update[0].email.clone_from(&email);
    speaker_to_update[0].phone_number.clone_from(&phone_number);

    sqlx::query_as::<Postgres, Speaker>(
        "UPDATE speakers
        SET name = $1, email = $2, phone_number = $3
        WHERE id = $3;",
    )
    .bind(name)
    .bind(email)
    .bind(phone_number)
    .bind(index)
    .fetch_all(db_pool)
    .await?;

    Ok(speaker_to_update)
}
