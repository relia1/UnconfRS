use crate::middleware::auth::AuthSessionLayer;
use crate::types::ApiStatusCode;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{response::Response, Json};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use std::error::Error;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
/// An enumeration of possible errors that can occur when working with sessions.
///
/// # Variants
/// - `DoesNotExist` - The session does not exist
pub enum SessionErr {
    #[error("Session {0} doesn't exist")]
    DoesNotExist(String),
    #[error("Session does not belong to user")]
    UnAuthorizedMutableAccess(String),
}

/// Struct representing an error that occurred when working with sessions.
///
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
#[derive(Debug, ToSchema)]
pub struct SessionError {
    pub status: ApiStatusCode,
    pub error: String,
}

/// Implements the `Serialize` trait for `SessionError`
///
/// This implementation serializes a `SessionError` into a JSON object with two properties:
/// `status` and `error`.
impl Serialize for SessionError {
    /// Serializes a `SessionError`
    ///
    /// The serialized JSON object will have two properties:
    /// - `status`: A string for the HTTP status code
    /// - `error`: A string describing the error
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status: String = self.status.to_string();
        let mut state = serializer.serialize_struct("SessionError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl SessionError {
    /// Creates a `Response` instance from a `StatusCode` and `SessionErr`.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code.
    /// - `error`: The `SessionErr` instance.
    ///
    /// # Returns
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: ApiStatusCode, error: Box<dyn Error>) -> Response {
        let error = SessionError {
            status,
            error: error.to_string(),
        };

        let http_status = StatusCode::from_u16(status.0)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (http_status, Json(error)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
/// Struct representing a session.
///
/// # Fields
/// - `Option<id>` - The ID of the session (optional)
/// - `title` - The title of the session
/// - `content` - The content of the session
/// - `votes` - The number of votes the session has
pub struct Session {
    pub id: Option<i32>,
    #[serde(skip_deserializing)]
    pub user_id: i32,
    pub title: String,
    pub content: String,
    #[serde(skip_deserializing)]
    pub votes: i32,
}

impl Session {
    /// Creates a new `Session` instance.
    ///
    /// # Parameters
    /// - `id`: The ID of the session (optional)
    /// - `title`: The title of the session
    /// - `content`: The content of the session
    /// - `votes`: The number of votes the session has
    ///
    /// # Returns
    /// A new `Session` instance
    pub fn new(id: Option<i32>, user_id: i32, title: &str, content: &str) -> Self {
        let title = title.into();
        let content = content.into();
        Self {
            id,
            user_id,
            title,
            content,
            votes: 0,
        }
    }
}

/// Implements the `IntoResponse` trait for `&Session` struct.
///
/// This implementation converts a `&Session` into an HTTP response. The response has a status code
/// of 200 OK and a JSON body containing the session data.
impl IntoResponse for &Session {
    /// Converts a `&Session` into an HTTP response.
    ///
    /// # Returns
    /// A `Response` object with a status code of 200 OK and a JSON body containing the session data.
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self)).into_response()
    }
}

/// Retrieves a list of sessions from the database.
///
/// This function retrieves a list of sessions from the database and returns them as a vector.
///
/// # Parameters
/// - `db_pool`: The database connection pool
///
/// # Returns
/// A vector of `Session` instances representing the sessions in the database or an error if the query
/// fails.
///
/// # Errors
/// If the query fails, a Box error is returned.
pub async fn get_all_sessions(db_pool: &Pool<Postgres>) -> Result<Vec<Session>, Box<dyn Error>> {
    let sessions: Vec<Session> = sqlx::query_as!(
        Session,
        r"
        SELECT * FROM sessions",
    )
        .fetch_all(db_pool)
        .await?;

    Ok(sessions)
}

/// Retrieves a session by its ID.
///
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `index`: The ID of the session
///
/// # Returns
/// The `Session` instance representing the session with the provided ID or an error
/// if the query fails.
///
/// # Errors
/// If the query fails, a Box error is returned.
pub async fn get(db_pool: &Pool<Postgres>, index: i32) -> Result<Session, Box<dyn Error>> {
    let session = sqlx::query_as!(
        Session,
        "SELECT * FROM sessions where id = $1",
        index,
    )
        .fetch_one(db_pool)
        .await?;

    Ok(session)
}

/// Adds a new session.
///
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `session`: The `Session` instance to add
///
/// # Returns
/// The ID of the newly added session or an error if the query fails.
///
/// # Errors
/// If the query fails, a Box error is returned.
pub async fn add(db_pool: &Pool<Postgres>, session: Session, auth_session: AuthSessionLayer) -> Result<i32, Box<dyn Error>> {
    let session_id = sqlx::query_scalar!(
        "INSERT INTO sessions (user_id, title, content, votes) VALUES ($1, $2, $3, $4) RETURNING id",
        auth_session.user.unwrap().id,
        session.title,
        session.content,
        session.votes,
    )
        .fetch_one(db_pool)
        .await?;

    Ok(session_id)
}

pub(crate) async fn is_users_resource(session: &Session, auth_session: &AuthSessionLayer) -> Result<bool, Box<dyn Error>> {
    if session.user_id == auth_session.user.clone().unwrap().id {
        Ok(true)
    } else {
        tracing::error!("cannot mutate other users resources");
        Err(Box::new(SessionErr::UnAuthorizedMutableAccess("User does not own this resource to mutate it".to_string())))
    }
}

/// Removes a session by its ID.
///
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `index`: The ID of the session to remove
///
/// # Returns
/// A `Result` indicating whether the session was removed successfully or an error if the query fails.
///
/// # Errors
/// If the query fails, a Box error is returned.
pub async fn delete(db_pool: &Pool<Postgres>, index: i32, auth_session: AuthSessionLayer) -> Result<(), Box<dyn Error>> {
    let session = sqlx::query_as!(
        Session,
        "SELECT * FROM sessions where id = $1",
        index,
    )
        .fetch_optional(db_pool)
        .await?;

    // The unwrap() here should be fine since by this point they have already been verified valid users
    let is_staff_or_admin = auth_session.backend.has_superuser_or_staff_perms(&auth_session.user.clone().unwrap()).await?;
    tracing::info!("Removing session: {:?}, is_staff_or_admin: {:?}", session, is_staff_or_admin);

    match session {
        Some(session) => {
            if is_staff_or_admin {
                sqlx::query!(
                    "DELETE FROM sessions WHERE id = $1;",
                    index,
                )
                    .bind(index)
                    .execute(db_pool)
                    .await?;
            } else {
                is_users_resource(&session, &auth_session).await?;
                sqlx::query!(
                    "DELETE FROM sessions WHERE id = $1 AND user_id = $2",
                    index,
                    auth_session.user.clone().unwrap().id
                )
                    .execute(db_pool)
                    .await?;
            }
        }
        None => {
            // In theory this shouldn't happen
            return Err(Box::new(SessionErr::DoesNotExist("Cannot find session to delete".to_string())));
        }
    }

    Ok(())
}

/// Updates a session by its ID.
///
/// # Parameters
/// - `index`: The ID of the session to update.
/// - `session`: The updated `Session` instance.
///
/// # Returns
/// The updated `Session` instance or an error if the query fails.
///
/// # Errors
/// If the query fails, a Box error is returned.
pub async fn update(
    db_pool: &Pool<Postgres>,
    index: i32,
    session: Session,
) -> Result<Session, Box<dyn Error>> {
    let title = session.title;
    let content = session.content;

    let mut session_to_update = get(db_pool, index).await?;
    session_to_update.title.clone_from(&title);
    session_to_update.content.clone_from(&content);

    sqlx::query!(
        "UPDATE sessions SET title = $1, content = $2 WHERE id = $3",
        title,
        content,
        index,
    )
        .execute(db_pool)
        .await?;

    Ok(session_to_update)
}


