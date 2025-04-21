use crate::middleware::auth::AuthSessionLayer;
use crate::types::ApiStatusCode;
use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Response, Json};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use std::error::Error;
use utoipa::ToSchema;

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
pub struct UserInfo {
    pub user_id: Option<i32>,
    pub name: String,
    pub email: String,
    pub phone_number: String,
}

/// An enumeration of possible errors that can occur when working with user_info.
///
/// This enum represents the possible errors that can occur when working with user_info.
///
/// # Variants
/// - `DoesNotExist` - The speaker does not exist
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum UserInfoErr {
    #[error("User {0} doesn't exist")]
    DoesNotExist(String),
}

/// Struct representing an error that occurred when working with user_info.
///
/// This struct represents an error that occurred when working with user_info.
///
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
#[derive(Debug, ToSchema)]
pub struct UserInfoError {
    pub status: ApiStatusCode,
    pub error: String,
}

/// Implements the `Serialize` trait for `SpeakerError`
///
/// This implementation serializes a `SpeakerError` into a JSON object with two properties: `status`
/// and `error`.
impl Serialize for UserInfoError {
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
        let mut state = serializer.serialize_struct("UserInfoError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl UserInfoError {
    /// Creates a `Response` instance from a `StatusCode` and `UserInfoErr`.
    ///
    /// This function creates a `Response` instance from a `StatusCode` and a `UserInfoErr`. The
    /// `UserInfoErr` is serialized into a JSON object with two properties: `status` and `error`.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code to return
    /// - `error`: The `UserInfoErr` error to return
    ///
    /// # Returns
    /// A `Response` instance with the provided status code and error.
    pub fn response(status: ApiStatusCode, error: Box<dyn Error>) -> Response {
        let error = UserInfoError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
}

impl UserInfo {
    /// Creates a new `UserInfo` instance.
    ///
    /// This function creates a new `UserInfo` instance with the provided user ID, name, email,
    /// and phone number.
    ///
    /// # Parameters
    /// - `Option<user_id>`: The ID of the user or None if the user is new
    /// - `name`: The name of the user
    /// - `email`: The email of the user
    /// - `phone_number`: The phone number of the user
    ///
    /// # Returns
    /// A new `UserInfo` instance
    pub fn new(user_id: Option<i32>, name: String, email: String, phone_number: String) -> Self {
        Self {
            user_id,
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
impl IntoResponse for &UserInfo {
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

/// Retrieves a list of user_info from the database.
///
/// This function retrieves a list of user_info from the database.
///
/// # Parameters
/// - `db_pool`: The database connection pool
///
/// # Returns
/// A vector of `Speaker` instances representing the user_info in the database or an error if the
/// query fails.
///
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn users_info_get(db_pool: &Pool<Postgres>) -> Result<Vec<UserInfo>, Box<dyn Error>> {
    let users_with_topics: Vec<UserInfo> = sqlx::query_as(
        r"
        SELECT * FROM user_info;",
    )
        .fetch_all(db_pool)
        .await?;

    Ok(users_with_topics)
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
pub async fn user_info_get(db_pool: &Pool<Postgres>, index: i32) -> Result<UserInfo, Box<dyn Error>> {
    let speaker = sqlx::query_as::<Postgres, UserInfo>(
        "SELECT user_id, name, email, \
        phone_number FROM user_info where id = $1",
    )
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
pub async fn user_info_add(
    db_pool: &Pool<Postgres>,
    user_info: UserInfo,
    auth_session: AuthSessionLayer,
) -> Result<i32, Box<dyn Error>> {
    tracing::trace!("\n\nauth_session: {:?}\n\n", auth_session.user);
    let (speaker_id,) = sqlx::query_as(
        "INSERT INTO user_info (name, email, phone_number, user_id) VALUES ($1, $2, $3, $4) RETURNING id",
    )
        .bind(user_info.name)
        .bind(user_info.email)
        .bind(user_info.phone_number)
        .bind(auth_session.user.unwrap().id)
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
pub async fn user_info_delete(db_pool: &Pool<Postgres>, index: i32) -> Result<(), Box<dyn Error>> {
    sqlx::query_as::<Postgres, UserInfo>(
        "DELETE FROM user_info
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
pub async fn user_info_update(
    db_pool: &Pool<Postgres>,
    index: i32,
    user_info: UserInfo,
) -> Result<UserInfo, Box<dyn Error>> {
    let user_id = user_info.user_id;
    let name = user_info.name;
    let email = user_info.email;
    let phone_number = user_info.phone_number;

    let mut user_info_to_update = user_info_get(db_pool, index).await?;
    tracing::debug!("speaker to update: {:?}", &user_info_to_update);
    user_info_to_update.user_id.clone_from(&user_id);
    user_info_to_update.name.clone_from(&name);
    user_info_to_update.email.clone_from(&email);
    user_info_to_update.phone_number.clone_from(&phone_number);

    sqlx::query_as::<Postgres, UserInfo>(
        "UPDATE user_info
        SET name = $1, email = $2, phone_number = $3
        WHERE user_id = $4;",
    )
        .bind(name)
        .bind(email)
        .bind(phone_number)
        .bind(user_id.unwrap())
        .fetch_all(db_pool)
        .await?;

    Ok(user_info_to_update)
}
