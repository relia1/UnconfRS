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

use crate::{speakers_model::*, UnconfData};

#[derive(OpenApi)]
#[openapi(
    paths(
        speakers,
        get_speaker,
        post_speaker,
        delete_speaker,
        update_speaker,
    ),
    components(
        schemas(SpeakerWithoutId, SpeakerError)
    ),
    tags(
        (name = "Speakers Server API", description = "Speakers Server API")
    )
)]
pub struct ApiDocSpeaker;

#[utoipa::path(
    get,
    path = "/api/v1/speakers",
    params(
        ("page" = i32, Query, description = "Page", minimum = 1),
        ("limit" = i32, Query, description = "Limit", minimum = 1)
    ),
    responses(
        (status = 200, description = "List speakers", body = Vec<Speaker>),
        (status = 404, description = "No speakers in that range")
    )
)]
pub async fn speakers(
    State(speakers): State<Arc<RwLock<UnconfData>>>,
    Query(params): Query<Pagination>,
) -> Response {
    let read_lock = speakers.read().await;
    match speaker_paginated_get(&read_lock.unconf_db, params.page, params.limit).await {
        Ok(res) => {
            Json(res).into_response()
        }
        Err(e) => {
            tracing::trace!("Paginated get error");
            SpeakerError::response(
                StatusCode::NOT_FOUND,
                Box::new(SpeakerErr::DoesNotExist(e.to_string())),
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/speakers/{id}",
    responses(
        (status = 200, description = "Return specified speaker", body = Speaker),
        (status = 404, description = "No speaker with this id", body = SpeakerError),
    )
)]
pub async fn get_speaker(
    State(speakers): State<Arc<RwLock<UnconfData>>>,
    Path(speaker_id): Path<i32>,
) -> Response {
    let read_lock = speakers.read().await;
    match speaker_get(&read_lock.unconf_db, speaker_id).await {
        Ok(speaker) => Json(speaker).into_response(),
        Err(e) => SpeakerError::response(StatusCode::NOT_FOUND, e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/speakers/add",
    request_body(
        content = inline(SpeakerWithoutId),
        description = "Speaker to add"
    ),
    responses(
        (status = 201, description = "Added speaker", body = ()),
        (status = 400, description = "Bad request", body = SpeakerError)
    )
)]
pub async fn post_speaker(
    State(speakers): State<Arc<RwLock<UnconfData>>>,
    Json(speaker): Json<SpeakerWithoutId>,
) -> Response {
    tracing::info!("post speaker!");
    let write_lock = speakers.write().await;
    match speaker_add(&write_lock.unconf_db, speaker).await {
        Ok(id) => {
            trace!("id: {:?}\n", id);
            //StatusCode::CREATED.into_response()
            let id_res = Json(format!("{{ \"id\": {} }}", id ));
            id_res.into_response()
        },
        Err(e) => SpeakerError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/speakers/{id}",
    responses(
        (status = 200, description = "Deleted speaker", body = ()),
        (status = 400, description = "Bad request", body = SpeakerError),
    )
)]
pub async fn delete_speaker(
    State(speakers): State<Arc<RwLock<UnconfData>>>,
    Path(speaker_id): Path<i32>,
) -> Response {
    tracing::info!("delete speaker");
    let write_lock = speakers.write().await;
    match speaker_delete(&write_lock.unconf_db, speaker_id).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => SpeakerError::response(StatusCode::BAD_REQUEST, e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/speakers/{id}",
    request_body(
        content = inline(Speaker),
        description = "Speaker to update"
    ),
    responses(
        (status = 200, description = "Updated speaker", body = ()),
        (status = 400, description = "Bad request", body = SpeakerError),
        (status = 404, description = "Speaker not found", body = SpeakerError),
        (status = 422, description = "Unprocessable entity", body = SpeakerError),
    )
)]
#[debug_handler]
pub async fn update_speaker(
    State(speakers): State<Arc<RwLock<UnconfData>>>,
    Path(speaker_id): Path<i32>,
    Json(speaker): Json<Speaker>,
) -> Response {
    let write_lock = speakers.write().await;
    match speaker_update(&write_lock.unconf_db, speaker_id, speaker).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => SpeakerError::response(StatusCode::BAD_REQUEST, e),
    }
}
