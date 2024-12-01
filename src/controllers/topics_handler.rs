use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::topics_model::{
    add, decrement_vote, delete, get, get_all_topics, increment_vote, update, Topic,
    TopicErr, TopicError,
};
use crate::StatusCode;
use askama_axum::IntoResponse;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Response;
use axum::Json;
use tracing::trace;
use crate::config::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/topics",
    params(
        ("page" = i32, Query, description = "Page", minimum = 1),
        ("limit" = i32, Query, description = "Limit", minimum = 1)
    ),
    responses(
        (status = 200, description = "List topics", body = Vec<Topic>),
        (status = 404, description = "No topics in that range")
    )
)]
/// Retrieves a list of topics
/// 
/// This function is a handler for the route `GET /api/v1/topics`. It retrieves a list of topics
/// from the database.
/// 
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// 
/// # Returns
/// `Response` with a status code of 200 OK and a JSON body containing the list of topics or an
/// error response if no topics are found.
/// 
/// # Errors
/// If an error occurs while retrieving the topics, a topic error response with a status code
/// of 404 Not Found is returned.
pub async fn topics(State(app_state): State<Arc<RwLock<AppState>>>,) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match get_all_topics(read_lock).await {
        Ok(res) => {
            Json(res).into_response()
        }
        Err(e) => TopicError::response(
            StatusCode::NOT_FOUND,
            Box::new(TopicErr::DoesNotExist(e.to_string())),
        ),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/topics/{id}",
    responses(
        (status = 200, description = "Return specified topic", body = Topic),
        (status = 404, description = "No topic with this id", body = TopicError),
    )
)]
/// Retrieves a topic by id
/// 
/// This function is a handler for the route `GET /api/v1/topics/{id}`. It retrieves a topic
/// from the database by its id.
/// 
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `topic_id` - The id of the topic to retrieve
/// 
/// # Returns
/// `Response` with a status code of 200 OK and a JSON body containing the topic or an error
/// response if the topic is not found.
/// 
/// # Errors
/// If an error occurs while retrieving the topic, a topic error response with a status code
/// of 404 Not Found is returned.
pub async fn get_topic(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(topic_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match get(read_lock, topic_id).await {
        Ok(topic) => Json(topic).into_response(),
        Err(e) => TopicError::response(StatusCode::NOT_FOUND, e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/topics/add",
    request_body(
        content = inline(Topic),
        description = "Topic to add"
    ),
    responses(
        (status = 201, description = "Added topic", body = ()),
        (status = 400, description = "Bad request", body = TopicError)
    )
)]
/// Adds a new topic.
/// 
/// This function is a handler for the route `POST /api/v1/topics/add`. It adds a new topic to the
/// database.
/// 
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `topic` - The topic to add
/// 
/// # Returns
/// `Response` with a status code of 201 Created and an empty body if the topic was added or an
/// error response if the topic could not be added.
/// 
/// # Errors
/// If an error occurs while adding the topic, a topic error response with a status code of 400
/// Bad Request is returned.
pub async fn post_topic(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(topic): Json<Topic>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match add(write_lock, topic).await {
        Ok(_) => {
            StatusCode::CREATED.into_response()
        }
        Err(e) => TopicError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/topics/{id}",
    responses(
        (status = 200, description = "Deleted topic", body = ()),
        (status = 400, description = "Bad request", body = TopicError),
    )
)]
/// Deletes a topic
/// 
/// This function is a handler for the route `DELETE /api/v1/topics/{id}`. It deletes a topic from
/// the database.
/// 
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `topic_id` - The id of the topic to delete
/// 
/// # Returns
/// `Response` with a status code of 200 OK if the topic was deleted or an error response if the
/// topic could not be deleted.
/// 
/// # Errors
/// If an error occurs while deleting the topic, a topic error response with a status code of
/// 400 Bad Request is returned.
pub async fn delete_topic(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(topic_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match delete(write_lock, topic_id).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => TopicError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/topics/{id}",
    request_body(
        content = inline(Topic),
        description = "Topic to update"
    ),
    responses(
        (status = 200, description = "Updated topic", body = ()),
        (status = 400, description = "Bad request", body = TopicError),
        (status = 404, description = "Topic not found", body = TopicError),
        (status = 422, description = "Unprocessable entity", body = TopicError),
    )
)]
#[debug_handler]
/// Updates a topic
/// 
/// This function is a handler for the route `PUT /api/v1/topics/{id}`. It updates a topic in the
/// database.
/// 
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `topic_id` - The id of the topic to update
/// - `topic` - The topic value to use for the update
/// 
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the topic was updated or an error
/// response if the topic could not be updated.
/// 
/// # Errors
/// If an error occurs while updating the topic, a topic error response with a status code of
/// 400 Bad Request is returned.
pub async fn update_topic(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(topic_id): Path<i32>,
    Json(topic): Json<Topic>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match update(write_lock, topic_id, topic).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => TopicError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/topics/{id}/increment",
    responses(
        (status = 200, description = "Updated topic", body = ()),
        (status = 400, description = "Bad request", body = TopicError),
        (status = 404, description = "Topic not found", body = TopicError),
        (status = 422, description = "Unprocessable entity", body = TopicError),
    )
)]
#[debug_handler]
/// Increments the vote count for a topic
/// 
/// This function is a handler for the route `PUT /api/v1/topics/{id}/increment`. It increments the
/// vote count for a topic in the database.
/// 
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `topic_id` - The id of the topic to increment the vote count for
/// 
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the topic was updated or an error
/// response if the topic vote could not be updated.
/// 
/// # Errors
/// If an error occurs while updating the topic vote, a topic error response with a status code of
/// 400 Bad Request is returned.
pub async fn add_vote_for_topic(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(topic_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match increment_vote(write_lock, topic_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => TopicError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/topics/{id}/decrement",
    responses(
        (status = 200, description = "Updated topic", body = ()),
        (status = 400, description = "Bad request", body = TopicError),
        (status = 404, description = "Topic not found", body = TopicError),
        (status = 422, description = "Unprocessable entity", body = TopicError),
    )
)]
#[debug_handler]
/// Decrements the vote count for a topic
/// 
/// This function is a handler for the route `PUT /api/v1/topics/{id}/decrement`. It decrements the
/// vote count for a topic in the database.
/// 
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `topic_id` - The id of the topic to decrement the vote count for
/// 
/// # Returns
/// `Response` with a status code of 200 OK and an empty body if the topic was updated or an error
/// response if the topic vote could not be updated.
/// 
/// # Errors
/// If an error occurs while updating the topic vote, a topic error response with a status code of
/// 400 Bad Request is returned.
pub async fn subtract_vote_for_topic(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(topic_id): Path<i32>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match decrement_vote(write_lock, topic_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => TopicError::response(StatusCode::BAD_REQUEST, e),
    }
}
