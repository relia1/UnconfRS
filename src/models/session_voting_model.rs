use crate::middleware::auth::AuthSessionLayer;
use crate::types::ApiStatusCode;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use sqlx::{Pool, Postgres};
use std::error::Error;
use utoipa::ToSchema;


#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
/// An enumeration of possible errors that can occur when working with sessions.
///
/// # Variants
/// - `NonExistentVote` - The `User` does not have a vote to remove from this session
/// - `AlreadyVotedForSession` - The `User` has already voted for the session
pub enum SessionVoteErr {
    #[error("Attempted to remove vote from Session {0} that didn't have a vote")]
    NonExistentVote(String),
    #[error("User has already voted for Session {0}")]
    AlreadyVotedForSession(String),
}

/// Struct representing an error that occurred when working with sessions.
///
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
#[derive(Debug, ToSchema)]
pub struct SessionVoteError {
    pub status: ApiStatusCode,
    pub error: String,
}

/// Implements the `Serialize` trait for `SessionVoteError`
///
/// This implementation serializes a `SessionVoteError` into a JSON object with two properties:
/// `status` and `error`.
impl Serialize for SessionVoteError {
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
        let mut state = serializer.serialize_struct("SessionVoteError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl SessionVoteError {
    /// Creates a `Response` instance from a `StatusCode` and `SessionErr`.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code.
    /// - `error`: The `SessionErr` instance.
    ///
    /// # Returns
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: ApiStatusCode, error: Box<dyn Error>) -> Response {
        let error = SessionVoteError {
            status,
            error: error.to_string(),
        };

        let http_status = StatusCode::from_u16(status.0)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (http_status, Json(error)).into_response()
    }
}


/// Adds a vote to a session
///
/// # Parameters
/// - `index`: The ID of the session to update.
///
/// # Returns
/// An empty `Result` if the vote was incremented successfully or an error if the query fails.
///
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn increment_vote(db_pool: &Pool<Postgres>, auth_session: AuthSessionLayer, index: i32) -> Result<Vec<i32>, Box<dyn Error>> {
    let user_id = auth_session.user.clone().unwrap().id;
    let mut sessions_user_voted_for = get_sessions_user_voted_for(db_pool, user_id).await?;

    if sessions_user_voted_for.contains(&index) {
        return Err(Box::new(SessionVoteErr::AlreadyVotedForSession(format!("Attempted to add vote to Session {index} that already had their vote"))));
    }

    sqlx::query(
        "INSERT INTO user_votes (user_id, session_id) VALUES ($1, $2)",
    )
        .bind(user_id)
        .bind(index)
        .execute(db_pool)
        .await?;

    sessions_user_voted_for.push(index);

    Ok(sessions_user_voted_for)
}

/// Removes a vote to a session
///
/// # Parameters
/// - `index`: The ID of the session to update.
///
/// # Returns
/// An empty `Result` if the vote was decremented successfully or an error if the query fails.
///
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn decrement_vote(db_pool: &Pool<Postgres>, auth_session: AuthSessionLayer, index: i32) -> Result<Vec<i32>, Box<dyn Error>> {
    let user_id = auth_session.user.clone().unwrap().id;
    let mut sessions_user_voted_for = get_sessions_user_voted_for(db_pool, user_id).await?;

    if !sessions_user_voted_for.contains(&index) {
        return Err(Box::new(SessionVoteErr::NonExistentVote(format!("Attempted to remove vote from Session {index} that didn't have their vote"))));
    }

    sqlx::query(
        "DELETE FROM user_votes WHERE user_id = $1 AND session_id = $2",
    )
        .bind(user_id)
        .bind(index)
        .execute(db_pool)
        .await?;

    sessions_user_voted_for.retain(|&session_id| session_id != index);

    Ok(sessions_user_voted_for)
}

pub async fn get_sessions_user_voted_for(db_pool: &Pool<Postgres>, user_id: i32) -> Result<Vec<i32>, Box<dyn Error>> {
    let (sessions_user_voted_for, ) = (sqlx::query_scalar(
        "SELECT session_id FROM user_votes WHERE user_id = $1",
    )
        .bind(user_id)
        .fetch_all(db_pool)
        .await?,);

    Ok(sessions_user_voted_for)
}