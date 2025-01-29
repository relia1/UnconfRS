use crate::types::ApiStatusCode;
use askama_axum::IntoResponse;
use axum::http::StatusCode;
use axum::{response::Response, Json};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use std::error::Error;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
/// An enumeration of possible errors that can occur when working with topics.
///
/// # Variants
/// - `DoesNotExist` - The topic does not exist
pub enum TopicErr {
    #[error("Topic {0} doesn't exist")]
    DoesNotExist(String),
}

/// Struct representing an error that occurred when working with topics.
///
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
#[derive(Debug, ToSchema)]
pub struct TopicError {
    pub status: ApiStatusCode,
    pub error: String,
}

/// Implements the `Serialize` trait for `TopicError`
///
/// This implementation serializes a `TopicError` into a JSON object with two properties:
/// `status` and `error`.
impl Serialize for TopicError {
    /// Serializes a `TopicError`
    ///
    /// The serialized JSON object will have two properties:
    /// - `status`: A string for the HTTP status code
    /// - `error`: A string describing the error
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status: String = self.status.to_string();
        let mut state = serializer.serialize_struct("TopicError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl TopicError {
    /// Creates a `Response` instance from a `StatusCode` and `TopicErr`.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code.
    /// - `error`: The `TopicErr` instance.
    ///
    /// # Returns
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: ApiStatusCode, error: Box<dyn Error>) -> Response {
        let error = TopicError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
/// Struct representing a topic.
///
/// # Fields
/// - `Option<id>` - The ID of the topic (optional)
/// - `speaker_id` - The ID of the speaker who created the topic
/// - `title` - The title of the topic
/// - `content` - The content of the topic
/// - `votes` - The number of votes the topic has
pub struct Topic {
    pub id: Option<i32>,
    pub speaker_id: i32,
    pub title: String,
    pub content: String,
    #[serde(skip_deserializing)]
    pub votes: i32,
}

impl Topic {
    /// Creates a new `Topic` instance.
    ///
    /// # Parameters
    /// - `id`: The ID of the topic (optional)
    /// - `speaker_id`: The ID of the speaker who created the topic
    /// - `title`: The title of the topic
    /// - `content`: The content of the topic
    /// - `votes`: The number of votes the topic has
    ///
    /// # Returns
    /// A new `Topic` instance
    pub fn new(id: Option<i32>, speaker_id: i32, title: &str, content: &str) -> Self {
        let title = title.into();
        let content = content.into();
        Self {
            id,
            speaker_id,
            title,
            content,
            votes: 0,
        }
    }
}

/// Implements the `IntoResponse` trait for `&Topic` struct.
///
/// This implementation converts a `&Topic` into an HTTP response. The response has a status code
/// of 200 OK and a JSON body containing the topic data.
impl IntoResponse for &Topic {
    /// Converts a `&Topic` into an HTTP response.
    ///
    /// # Returns
    /// A `Response` object with a status code of 200 OK and a JSON body containing the topic data.
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self)).into_response()
    }
}

/// Retrieves a list of topics from the database.
///
/// This function retrieves a list of topics from the database and returns them as a vector.
///
/// # Parameters
/// - `db_pool`: The database connection pool
///
/// # Returns
/// A vector of `Topic` instances representing the topics in the database or an error if the query
/// fails.
///
/// # Errors
/// If the query fails, a Box error is returned.
pub async fn get_all_topics(db_pool: &Pool<Postgres>) -> Result<Vec<Topic>, Box<dyn Error>> {
    let topics: Vec<Topic> = sqlx::query_as(
        r#"
        SELECT * FROM topics"#,
    )
        .fetch_all(db_pool)
        .await?;

    Ok(topics)
}

/// Retrieves a topic by its ID.
///
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `index`: The ID of the topic
///
/// # Returns
/// The `Topic` instance representing the topic with the provided ID or an error
/// if the query fails.
///
/// # Errors
/// If the query fails, a Box error is returned.
pub async fn get(db_pool: &Pool<Postgres>, index: i32) -> Result<Topic, Box<dyn Error>> {
    let topic = sqlx::query_as::<Postgres, Topic>("SELECT * FROM topics where id = $1")
        .bind(index)
        .fetch_one(db_pool)
        .await?;

    Ok(topic)
}

/// Adds a new topic.
///
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `topic`: The `Topic` instance to add
///
/// # Returns
/// The ID of the newly added topic or an error if the query fails.
///
/// # Errors
/// If the query fails, a Box error is returned.
pub async fn add(db_pool: &Pool<Postgres>, topic: Topic) -> Result<i32, Box<dyn Error>> {
    let (topic_id,) = sqlx::query_as(
        "INSERT INTO topics (speaker_id, title, content, votes) VALUES ($1, $2, $3, $4) RETURNING id",
        )
            .bind(topic.speaker_id)
            .bind(topic.title)
            .bind(topic.content)
            .bind(topic.votes)
            .fetch_one(db_pool)
            .await?;

    Ok(topic_id)
}

/// Removes a topic by its ID.
///
/// # Parameters
/// - `db_pool`: The database connection pool
/// - `index`: The ID of the topic to remove
///
/// # Returns
/// A `Result` indicating whether the topic was removed successfully or an error if the query fails.
///
/// # Errors
/// If the query fails, a Box error is returned.
pub async fn delete(db_pool: &Pool<Postgres>, index: i32) -> Result<(), Box<dyn Error>> {
    sqlx::query_as::<Postgres, Topic>(
        "DELETE FROM topics
        WHERE id = $1;",
    )
        .bind(index)
        .fetch_all(db_pool)
        .await?;

    Ok(())
}

/// Updates a topic by its ID.
///
/// # Parameters
/// - `index`: The ID of the topic to update.
/// - `topic`: The updated `Topic` instance.
///
/// # Returns
/// The updated `Topic` instance or an error if the query fails.
///
/// # Errors
/// If the query fails, a Box error is returned.
pub async fn update(
    db_pool: &Pool<Postgres>,
    index: i32,
    topic: Topic,
) -> Result<Topic, Box<dyn Error>> {
    let title = topic.title;
    let content = topic.content;

    let mut topic_to_update = get(db_pool, index).await?;
    topic_to_update.title.clone_from(&title);
    topic_to_update.content.clone_from(&content);

    sqlx::query_as::<Postgres, Topic>(
        "UPDATE topics
        SET title = $1, content = $2
        WHERE id = $3;",
    )
        .bind(title)
        .bind(content)
        .bind(index)
        .fetch_all(db_pool)
        .await?;

    Ok(topic_to_update)
}

/// Adds a vote to a topic
///
/// # Parameters
/// - `index`: The ID of the topic to update.
///
/// # Returns
/// An empty `Result` if the vote was incremented successfully or an error if the query fails.
///
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn increment_vote(db_pool: &Pool<Postgres>, index: i32) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        "UPDATE topics
        SET votes = votes + 1
        WHERE id = $1;",
    )
        .bind(index)
        .fetch_all(db_pool)
        .await?;

    Ok(())
}

/// Removes a vote to a topic
///
/// # Parameters
/// - `index`: The ID of the topic to update.
///
/// # Returns
/// An empty `Result` if the vote was decremented successfully or an error if the query fails.
///
/// # Errors
/// If the query fails, a boxed error is returned.
pub async fn decrement_vote(db_pool: &Pool<Postgres>, index: i32) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        "UPDATE topics
        SET votes = votes - 1
        WHERE id = $1;",
    )
        .bind(index)
        .fetch_all(db_pool)
        .await?;

    Ok(())
}
