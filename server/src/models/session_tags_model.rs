use crate::middleware::auth::AuthSessionLayer;
use crate::models::sessions_model;
use crate::models::sessions_model::is_users_resource;
use crate::models::tags_model::Tag;
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
pub enum SessionTagErr {
    #[error("Attempted to perform action with tag that doesn't exist")]
    NonExistentTag(String),
    #[error("Tag already applied to Session {0}")]
    AlreadyAppliedTagForSession(String),
    #[error("User does not have access to mutating session tags")]
    UnAuthorizedMutableAccess(String),
}

/// Struct representing an error that occurred when working with sessions.
///
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
#[derive(Debug, ToSchema)]
pub struct SessionTagError {
    pub status: ApiStatusCode,
    pub error: String,
}

/// Implements the `Serialize` trait for `SessionVoteError`
///
/// This implementation serializes a `SessionVoteError` into a JSON object with two properties:
/// `status` and `error`.
impl Serialize for SessionTagError {
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

impl SessionTagError {
    /// Creates a `Response` instance from a `StatusCode` and `SessionErr`.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code.
    /// - `error`: The `SessionErr` instance.
    ///
    /// # Returns
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: ApiStatusCode, error: Box<dyn Error>) -> Response {
        let error = SessionTagError {
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
pub async fn add_session_tag(db_pool: &Pool<Postgres>, auth_session: AuthSessionLayer, session_id: i32, tag_id: i32) -> Result<Vec<Tag>, Box<dyn Error>> {
    // The unwrap() here should be fine since by this point they have already been verified valid users
    let is_staff_or_admin = auth_session
        .backend
        .has_superuser_or_staff_perms(&auth_session.user.clone().unwrap())
        .await?;

    let session = sessions_model::get(db_pool, session_id).await?;

    tracing::info!("Adding tag with id {} to session: {:?}, is_staff_or_admin: {:?}", tag_id, session_id, is_staff_or_admin);

    let current_tags = get_tags_for_session(db_pool, session_id).await?;

    if current_tags.iter().any(|tag| tag.id == tag_id) {
        return Err(Box::new(SessionTagErr::AlreadyAppliedTagForSession(
            format!("Attempted to add tag with id: {tag_id} to Session {session_id} that already had that tag")
        )));
    }

    let looked_up_tag_id = sqlx::query_scalar!(
        "SELECT id FROM tags WHERE id = $1",
        tag_id
    )
        .fetch_optional(db_pool)
        .await?;

    match looked_up_tag_id {
        Some(id) => id,
        None => {
            return Err(Box::new(SessionTagErr::NonExistentTag(tag_id.to_string())));
        }
    };

    if is_staff_or_admin {
        sqlx::query!(
            "INSERT INTO session_tags (session_id, tag_id) VALUES ($1, $2)",
            session_id,
            tag_id,
        )
            .execute(db_pool)
            .await?;
    } else {
        is_users_resource(&session, &auth_session).await?;
        sqlx::query!(
            "INSERT INTO session_tags (session_id, tag_id) VALUES ($1, $2)",
            session_id,
            tag_id,
        )
            .execute(db_pool)
            .await?;
    }

    get_tags_for_session(db_pool, session_id).await
}

pub async fn remove_session_tag(
    db_pool: &Pool<Postgres>,
    auth_session: AuthSessionLayer,
    session_id: i32,
    tag_id: i32,
) -> Result<Vec<Tag>, Box<dyn Error>> {
    // The unwrap() here should be fine since by this point they have already been verified valid users
    let is_staff_or_admin = auth_session
        .backend
        .has_superuser_or_staff_perms(&auth_session.user.clone().unwrap())
        .await?;

    let session = sessions_model::get(db_pool, session_id).await?;

    tracing::info!("Removing tag with id {} from session: {:?}, is_staff_or_admin: {:?}", tag_id, session_id, is_staff_or_admin);

    // Get current tags for the session
    let current_tags = get_tags_for_session(db_pool, session_id).await?;

    // Check if tag is currently applied
    if !current_tags.iter().any(|tag| tag.id == tag_id) {
        return Err(Box::new(SessionTagErr::NonExistentTag(
            format!("Attempted to remove tag with id: {tag_id} from Session {session_id} that didn't have that tag")
        )));
    }

    if is_staff_or_admin {
        sqlx::query!(
            "DELETE FROM session_tags
             WHERE session_id = $1 AND tag_id = $2",
            session_id,
            tag_id,
        )
            .execute(db_pool)
            .await?;
    } else {
        is_users_resource(&session, &auth_session).await?;
        sqlx::query!(
            "DELETE FROM session_tags
             WHERE session_id = $1 AND tag_id = $2",
            session_id,
            tag_id,
        )
            .execute(db_pool)
            .await?;
    }

    get_tags_for_session(db_pool, session_id).await
}

/// Updates a session's tag
///
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `auth_session`: Authentication session for authorization
/// - `session_id`: The ID of the session to update
/// - `old_tag_id`: The ID of the tag to replace
/// - `new_tag_id`: The ID of the new tag
///
/// # Returns
/// The updated list of tags for the session or an error if the operation fails.
///
/// # Errors
/// If the operation fails due to authorization, old tag not found, new tag already applied, etc.
pub async fn update_session_tag(
    db_pool: &Pool<Postgres>,
    auth_session: AuthSessionLayer,
    session_id: i32,
    old_tag_id: i32,
    new_tag_id: i32,
) -> Result<Vec<Tag>, Box<dyn Error>> {
    // The unwrap() here should be fine since by this point they have already been verified valid users
    let is_staff_or_admin = auth_session
        .backend
        .has_superuser_or_staff_perms(&auth_session.user.clone().unwrap())
        .await?;

    let session = sessions_model::get(db_pool, session_id).await?;

    tracing::info!("Updating tag for session: {:?}, changing tag {} to {}, is_staff_or_admin: {:?}", 
                  session_id, old_tag_id, new_tag_id, is_staff_or_admin);

    // Get current tags for the session
    let current_tags = get_tags_for_session(db_pool, session_id).await?;

    // Check if old tag is currently applied
    if !current_tags.iter().any(|tag| tag.id == old_tag_id) {
        return Err(Box::new(SessionTagErr::NonExistentTag(
            format!("Session {session_id} does not have tag with id: {old_tag_id}")
        )));
    }

    // Check if new tag is already applied (and is different from old tag)
    if old_tag_id != new_tag_id && current_tags.iter().any(|tag| tag.id == new_tag_id) {
        return Err(Box::new(SessionTagErr::AlreadyAppliedTagForSession(
            format!("Session {session_id} already has tag with id: {new_tag_id}")
        )));
    }

    // Verify the new tag exists
    let looked_up_tag_id = sqlx::query_scalar!(
        "SELECT id FROM tags WHERE id = $1",
        new_tag_id
    )
        .fetch_optional(db_pool)
        .await?;

    match looked_up_tag_id {
        Some(_id) => {}
        None => {
            return Err(Box::new(SessionTagErr::NonExistentTag(new_tag_id.to_string())));
        }
    };

    if is_staff_or_admin {
        // Update the tag
        sqlx::query!(
            "UPDATE session_tags SET tag_id = $1 WHERE session_id = $2 AND tag_id = $3",
            new_tag_id,
            session_id,
            old_tag_id,
        )
            .execute(db_pool)
            .await?;
    } else {
        is_users_resource(&session, &auth_session).await?;
        // Update the tag
        sqlx::query!(
            "UPDATE session_tags SET tag_id = $1 WHERE session_id = $2 AND tag_id = $3",
            new_tag_id,
            session_id,
            old_tag_id,
        )
            .execute(db_pool)
            .await?;
    }

    get_tags_for_session(db_pool, session_id).await
}

pub async fn get_tags_for_session(db_pool: &Pool<Postgres>, session_id: i32) -> Result<Vec<Tag>, Box<dyn Error>> {
    let session_tags = sqlx::query_as!(
        Tag,
        r#"
        SELECT T.id, T.tag_name
        FROM session_tags ST
        JOIN tags T ON ST.tag_id = T.id
        WHERE ST.session_id = $1
        "#,
        session_id
    )
        .fetch_all(db_pool)
        .await?;

    Ok(session_tags)
}

pub async fn get_all_tags(db_pool: &Pool<Postgres>) -> Result<Vec<Tag>, Box<dyn Error>> {
    let tags = sqlx::query_as!(
        Tag,
        "SELECT * FROM tags"
    )
        .fetch_all(db_pool)
        .await?;

    Ok(tags)
}