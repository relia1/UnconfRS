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
use utoipa::OpenApi;
use crate::config::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        topics,
        get_topic,
        post_topic,
        delete_topic,
        update_topic,
    ),
    components(
        schemas(Topic, TopicError)
    ),
    tags(
        (name = "Topics Server API", description = "Topics Server API")
    )
)]
pub struct ApiDoc;

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
pub async fn topics(State(app_state): State<Arc<RwLock<AppState>>>,) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match get_all_topics(read_lock).await {
        Ok(res) => {
            trace!("Retrieved {} topics", res.len());
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
pub async fn post_topic(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(topic): Json<Topic>,
) -> Response {
    tracing::info!("post topic!");
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    match add(write_lock, topic).await {
        Ok(id) => {
            trace!("id: {:?}\n", id);
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
pub async fn delete_topic(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(topic_id): Path<i32>,
) -> Response {
    tracing::info!("delete topic");
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
