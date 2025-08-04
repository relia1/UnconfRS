use crate::middleware::auth::AuthSessionLayer;
use crate::types::ApiStatusCode;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use std::error::Error;
use utoipa::ToSchema;


#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
/// An enumeration of possible errors that can occur when working with sessions.
///
/// # Variants
/// - `NonExistentVote` - The `User` does not have a vote to remove from this session
/// - `AlreadyVotedForSession` - The `User` has already voted for the session
pub enum TagErr {
    #[error("Tag with ID {0} not found")]
    TagNotFound(i32),
    #[error("Tag with name '{0}' not found")]
    TagNotFoundByName(String),
    #[error("Tag with name '{0}' already exists")]
    TagAlreadyExists(String),
    #[error("User does not have access to manage tags")]
    UnAuthorizedAccess(String),
    #[error("Unexpected error during query '{0}'")]
    UnexpectedError(String),
}

/// Struct representing an error that occurred when working with sessions.
///
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
#[derive(Debug, ToSchema)]
pub struct TagError {
    pub status: ApiStatusCode,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct Tag {
    pub id: i32,
    pub tag_name: String,
}

impl IntoResponse for &Tag {
    /// Converts a `&Session` into an HTTP response.
    ///
    /// # Returns
    /// A `Response` object with a status code of 200 OK and a JSON body containing the session data.
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self)).into_response()
    }
}

/// Implements the `Serialize` trait for `SessionVoteError`
///
/// This implementation serializes a `SessionVoteError` into a JSON object with two properties:
/// `status` and `error`.
impl Serialize for TagError {
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

impl TagError {
    /// Creates a `Response` instance from a `StatusCode` and `SessionErr`.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code.
    /// - `error`: The `SessionErr` instance.
    ///
    /// # Returns
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: ApiStatusCode, error: Box<dyn Error>) -> Response {
        let error = TagError {
            status,
            error: error.to_string(),
        };

        let http_status = StatusCode::from_u16(status.0)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (http_status, Json(error)).into_response()
    }
}

/// Gets all available tags
///
/// # Parameters
/// - `db_pool`: Database connection pool
///
/// # Returns
/// A `Vec<Tag>` of all available tags
///
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn get_all_tags(db_pool: &Pool<Postgres>) -> Result<Vec<Tag>, Box<dyn Error>> {
    let tags = sqlx::query_as!(
        Tag,
        "SELECT * FROM tags"
    )
        .fetch_all(db_pool)
        .await?;

    Ok(tags)
}

/// Get tag by its ID
///
/// # Parameters
/// - `db_pool`: Database connection pool
/// - `tag_id`: The ID of the tag to get
///
/// # Returns
/// The `Tag` if found
///
/// # Errors
/// Returns `TagNotFound` if there isn't a tag with the specified ID
pub async fn get_tag_by_id(db_pool: &Pool<Postgres>, tag_id: i32) -> Result<Tag, Box<dyn Error>> {
    let tag = sqlx::query_as!(
        Tag,
        "SELECT * FROM tags WHERE id = $1",
        tag_id,
    )
        .fetch_optional(db_pool)
        .await?;

    match tag {
        Some(tag) => Ok(tag),
        None => Err(Box::new(TagErr::TagNotFound(tag_id))),
    }
}

/// Get tag by its name
///
/// # Parameters
/// - `db_pool`: Database connection pool
/// - `tag_name`: The name of the tag to get
///
/// # Returns
/// The `Tag` if found
///
/// # Errors
/// Returns `TagNotFound` if there isn't a tag with the specified name
pub async fn get_tag_by_name(db_pool: &Pool<Postgres>, tag_name: &str) -> Result<Tag, Box<dyn Error>> {
    let tag = sqlx::query_as!(
        Tag,
        "SELECT * FROM tags WHERE tag_name = $1",
        tag_name,
    )
        .fetch_optional(db_pool)
        .await?;

    match tag {
        Some(tag) => Ok(tag),
        None => Err(Box::new(TagErr::TagNotFoundByName(tag_name.to_string()))),
    }
}

/// Create a new tag
///
/// # Parameters
/// - `db_pool`: Database connection pool
/// - `auth_session`: Authentication session containing user information
/// - `tag_name`: The name of the tag to create
///
/// # Returns
/// The newly created `Tag`
///
/// # Errors
/// Returns an error if:
/// - User isn't authorized to create tags
/// - The tag being created already exists
/// - Database query fails
pub async fn create_tag(db_pool: &Pool<Postgres>, auth_session: AuthSessionLayer, tag_name: &str) -> Result<Tag, Box<dyn Error>> {
    // The unwrap() here should be fine since by this point they have already been verified valid users
    let is_staff_or_admin = auth_session
        .backend
        .has_superuser_or_staff_perms(&auth_session.user.clone().unwrap())
        .await?;

    if !is_staff_or_admin {
        return Err(Box::new(TagErr::UnAuthorizedAccess(
            format!("Attempted to create tag: {tag_name}")
        )));
    }

    if get_tag_by_name(db_pool, tag_name).await.is_ok() {
        return Err(Box::new(TagErr::TagAlreadyExists(tag_name.to_string())));
    }

    let tag = sqlx::query_as!(
        Tag,
        "INSERT INTO tags (tag_name) VALUES ($1) RETURNING *",
        tag_name,
    )
        .fetch_one(db_pool)
        .await?;

    tracing::info!("Created new tag: {} with ID: {}", tag_name, tag.id);

    Ok(tag)
}

/// Updates an existing tag
///
/// # Parameters
/// - `db_pool`: Database connection pool
/// - `auth_session`: Authentication session containing user information
/// - `tag_id`: The ID of the tag to update
/// - `new_tag_name`: The new name for the tag
///
/// # Returns
/// The updated `Tag`
///
/// # Errors
/// Returns an error if:
/// - User is not authorized to update tags
/// - Tag with the given ID doesn't exist
/// - New tag name already exists
/// - Database query fails
pub async fn update_tag(
    db_pool: &Pool<Postgres>,
    auth_session: AuthSessionLayer,
    tag_id: i32,
    new_tag_name: &str,
) -> Result<Tag, Box<dyn Error>> {
    // The unwrap() here should be fine since by this point they have already been verified valid users
    let is_staff_or_admin = auth_session
        .backend
        .has_superuser_or_staff_perms(&auth_session.user.clone().unwrap())
        .await?;

    if !is_staff_or_admin {
        return Err(Box::new(TagErr::UnAuthorizedAccess(
            format!("Attempted to update tag with ID: {tag_id}")
        )));
    }

    // Verify the tag exists
    let _ = get_tag_by_id(db_pool, tag_id).await?;

    // Check if new name already exists and if it is not the same ID as the original return an Err
    if let Ok(existing_tag_with_name) = get_tag_by_name(db_pool, new_tag_name).await {
        if existing_tag_with_name.id != tag_id {
            return Err(Box::new(TagErr::TagAlreadyExists(new_tag_name.to_string())));
        }
    }

    let updated_tag = sqlx::query_as!(
        Tag,
        "UPDATE tags SET tag_name = $1 WHERE id = $2 RETURNING *",
        new_tag_name,
        tag_id
    )
        .fetch_one(db_pool)
        .await?;

    tracing::info!("Updated tag ID: {} to name: {}", tag_id, new_tag_name);

    Ok(updated_tag)
}

/// Deletes a tag
///
/// # Parameters
/// - `db_pool`: Database connection pool
/// - `auth_session`: Authentication session containing user information
/// - `tag_id`: The ID of the tag to delete
///
/// # Returns
/// Ok(()) if successful
///
/// # Errors
/// Returns an error if:
/// - User is not authorized to delete tags
/// - Tag doesn't exist
/// - Database query fails
///
/// # Note
/// This will also remove all session_tags relationships due to CASCADE DELETE
pub async fn delete_tag(
    db_pool: &Pool<Postgres>,
    auth_session: AuthSessionLayer,
    tag_id: i32,
) -> Result<(), Box<dyn Error>> {
    // The unwrap() here should be fine since by this point they have already been verified valid users
    let is_staff_or_admin = auth_session
        .backend
        .has_superuser_or_staff_perms(&auth_session.user.clone().unwrap())
        .await?;

    if !is_staff_or_admin {
        return Err(Box::new(TagErr::UnAuthorizedAccess(
            format!("Attempted to delete tag with ID: {tag_id}")
        )));
    }

    let tag = get_tag_by_id(db_pool, tag_id).await?;
    let tag_name = tag.tag_name.to_string();

    let rows_affected = sqlx::query!(
        "DELETE FROM tags WHERE id = $1",
        tag_id
    )
        .execute(db_pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(Box::new(TagErr::UnexpectedError(
            format!("Deleting tag name: {tag_name} ID: {tag_id}")
        )));
    }

    tracing::info!("Deleted tag: {} with ID: {}", tag.tag_name, tag_id);

    Ok(())
}