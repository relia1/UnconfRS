use std::sync::Arc;
use tokio::sync::RwLock;

use askama_axum::IntoResponse;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Response;
use axum::Json;
use axum::debug_handler;
use tracing::trace;
use utoipa::OpenApi;
use axum::extract::Path;
use crate::pagination::Pagination;
use crate::StatusCode;

use crate::topics_model::*;
use crate::UnconfData;

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
        schemas(TopicWithoutId, TopicError)
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
pub async fn topics(
    State(topics): State<Arc<RwLock<UnconfData>>>,
    Query(params): Query<Pagination>,
) -> Response {
    //let page = params.page;
    //let limit = params.limit;

    let read_lock = topics.read().await;
    match paginated_get(&read_lock.unconf_db, params.page, params.limit).await {
        Ok(res) => {
            tracing::trace!("Retrieved {} topics", res.len());
            Json(res).into_response()
        }
        Err(e) => {
            tracing::trace!("Paginated get error");
            TopicError::response(
                StatusCode::NOT_FOUND,
                Box::new(TopicErr::DoesNotExist(e.to_string())),
            )
        }
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
    State(topics): State<Arc<RwLock<UnconfData>>>,
    Path(topic_id): Path<i32>,
) -> Response {
    let read_lock = topics.read().await;
    match get(&read_lock.unconf_db, topic_id).await {
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
    State(topics): State<Arc<RwLock<UnconfData>>>,
    Json(topic): Json<TopicWithoutId>,
) -> Response {
    tracing::info!("post topic!");
    let write_lock = topics.write().await;
    match add(&write_lock.unconf_db, topic).await {
        Ok(id) => {
            trace!("id: {:?}\n", id);
            StatusCode::CREATED.into_response()
        },
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
    State(topics): State<Arc<RwLock<UnconfData>>>,
    Path(topic_id): Path<i32>,
) -> Response {
    tracing::info!("delete topic");
    let write_lock = topics.write().await;
    match delete(&write_lock.unconf_db, topic_id).await {
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
    State(topics): State<Arc<RwLock<UnconfData>>>,
    Path(topic_id): Path<i32>,
    Json(topic): Json<Topic>,
) -> Response {
    let write_lock = topics.write().await;
    match update(&write_lock.unconf_db, topic_id, topic).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => TopicError::response(StatusCode::BAD_REQUEST, e),
    }
}
