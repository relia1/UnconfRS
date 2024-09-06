use std::error::Error;

use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Response, Json};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Pool, Postgres};
use utoipa::{
    openapi::{ObjectBuilder, RefOr, Schema, SchemaType},
    ToSchema,
};

/// An enumeration of errors that may occur
#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
pub enum TopicErr {
    #[error("Topic {0} doesn't exist")]
    DoesNotExist(String),
}

/// struct that represents a Topic error, but include a `StatusCode`
/// in addition to a `TopicErr`
#[derive(Debug)]
pub struct TopicError {
    pub status: StatusCode,
    pub error: String,
}

/// Implements `ToSchema` trait for `TopicError` generating a JSON schema
/// for the error type
impl<'s> ToSchema<'s> for TopicError {
    /// Returns a JSON schema for `TopicError`
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
                "status":"404","error":"no topic"
            })))
            .into();
        ("TopicError", sch)
    }
}

/// Implements the `Serialize` trait for `TopicError`
impl Serialize for TopicError {
    /// Serializes a `TopicError`
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
    ///
    /// * `status`: The HTTP status code.
    /// * `error`: The `TopicErr` instance.
    ///
    /// # Returns
    ///
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: StatusCode, error: Box<dyn Error>) -> Response {
        let error = TopicError {
            status,
            error: error.to_string(),
        };
        (status, Json(error)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
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
    ///
    /// * `id`: ID for the topic.
    /// * `title`: The title of the topic.
    /// * `content`: The content of the topic.
    ///
    /// # Returns
    ///
    /// A new `Topic` instance with the provided parameters.
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

impl IntoResponse for &Topic {
    /// Converts a `&Topic` into an HTTP response.
    ///
    /// # Returns
    ///
    /// A `Response` object with a status code of 200 OK and a JSON body containing the topic data.
    fn into_response(self) -> Response {
        tracing::info!("{:?}", &self);
        (StatusCode::OK, Json(&self)).into_response()
    }
}

/// Retrieves a list of topics from the topic bank.
///
/// # Parameters
///
/// # Pooled db connection
///
/// # Returns
///
/// A vector of Topic's
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
///
/// * `index`: The ID of the topic.
///
/// # Returns
///
/// A reference to the `Topic` instance with the specified ID, or a `TopicErr` error if the topic does not exist.
pub async fn get(db_pool: &Pool<Postgres>, index: i32) -> Result<Vec<Topic>, Box<dyn Error>> {
    let mut topic_vec = vec![];
    let topic = sqlx::query_as::<Postgres, Topic>("SELECT * FROM topics where id = $1")
        .bind(index)
        .fetch_one(db_pool)
        .await?;

    // topic_vec.push(<Topic as std::convert::From<PgRow>>::from(topic));
    topic_vec.push(topic);
    Ok(topic_vec)
}

/// Adds a new topic.
///
/// # Parameters
///
/// * `topic`: The `Topic` to add to the topic bank.
///
/// # Returns
///
/// A `Result` indicating whether the topic was added successfully.
/// If the topic already exists, returns a `TopicErr` error.
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
///
/// * `index`: The ID of the topic.
///
/// # Returns
///
/// A `Result` indicating whether the topic was removed successfully.
/// If the topic does not exist, returns a `TopicErr` error.
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
///
/// * `index`: The ID of the topic to update.
/// * `topic`: The updated `Topic` instance.
///
/// # Returns
///
/// A `Result` indicating whether the topic was updated successfully.
/// If the topic does not exist or is unprocessable, returns a `TopicErr` error.
/// If successful, returns a `StatusCode` of 200.
pub async fn update(
    db_pool: &Pool<Postgres>,
    index: i32,
    topic: Topic,
) -> Result<Vec<Topic>, Box<dyn Error>> {
    let title = topic.title;
    let content = topic.content;

    let mut topic_to_update = get(db_pool, index).await?;
    topic_to_update[0].title.clone_from(&title);
    topic_to_update[0].content.clone_from(&content);

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
///
/// * `index`: The ID of the topic to update.
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
///
/// * `index`: The ID of the topic to update.
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
