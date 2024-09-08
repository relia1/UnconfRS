extern crate serde_json;
extern crate thiserror;
extern crate tracing;
mod config;
mod controllers;
mod db_config;
mod models;

use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use config::*;
use serde::Deserialize;
use std::error::Error;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace,
};
use tracing_subscriber::{fmt, EnvFilter};

// use askama_axum::Template;
use askama_axum::Template;
//use askama::Template;
use utoipa::OpenApi;

use crate::controllers::room_handler::{delete_room, post_rooms, rooms, ApiDocRooms};
use crate::controllers::schedule_handler::{
    clear, generate, get_schedule, post_schedule, schedules, update_schedule, ApiDocSchedule,
};
use crate::controllers::speakers_handler::{
    delete_speaker, get_speaker, post_speaker, speakers, update_speaker, ApiDocSpeaker,
};
use crate::controllers::timeslot_handler::{update_timeslot, ApiDocTimeslot};
use crate::controllers::topics_handler::{
    add_vote_for_topic, delete_topic, get_topic, post_topic, subtract_vote_for_topic, topics,
    update_topic, ApiDoc,
};
use crate::models::room_model::{rooms_get, Room};
use crate::models::schedule_model::{schedules_get, Schedule};
use crate::models::speakers_model::Speaker;
use crate::models::topics_model::{get_all_topics, Topic};
use sqlx::{FromRow, Pool, Postgres};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{self, fs::read_to_string, sync::RwLock};
use tower_http::compression::CompressionLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

async fn handler_404() -> Response {
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

// handler to load assets
async fn asset_handler(path: Path<String>) -> String {
    let path = path.to_string();
    if path.contains(".js") {
        let formatted_path = format!("../scripts/{}", path);
        read_to_string(formatted_path).await.unwrap()
    } else {
        let formatted_path = format!("../styles/{}", path);
        read_to_string(formatted_path).await.unwrap()
    }
}

#[tokio::main]
async fn main() {
    // Setup formatting and environment for trace
    let fmt_layer = fmt::layer().with_file(true).with_line_number(true).pretty();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust

    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new())
        .on_response(trace::DefaultOnResponse::new());

    // Connect to database
    let topics_db = Arc::new(RwLock::new(UnconfData::new().await.unwrap()));

    // routes with their handlers
    let apis = Router::new()
        .route("/topics", get(topics))
        .route("/topics/:id", get(get_topic))
        .route("/topics/add", post(post_topic))
        .route("/topics/:id", delete(delete_topic))
        .route("/topics/:id", put(update_topic))
        .route("/topics/:id/increment", put(add_vote_for_topic))
        .route("/topics/:id/decrement", put(subtract_vote_for_topic))
        .route("/rooms", get(rooms))
        .route("/rooms/add", post(post_rooms))
        .route("/rooms/:id", delete(delete_room))
        .route("/speakers", get(speakers))
        .route("/speakers/:id", get(get_speaker))
        .route("/speakers/add", post(post_speaker))
        .route("/speakers/:id", delete(delete_speaker))
        .route("/speakers/:id", put(update_speaker))
        .route("/schedules", get(schedules))
        .route("/schedules/:id", get(get_schedule))
        .route("/schedules/:id", put(update_schedule))
        .route("/schedules/add", post(post_schedule))
        .route("/schedules/generate", post(generate))
        .route("/schedules/clear", post(clear))
        .route("/timeslots/:id", put(update_timeslot));

    // handy openai auto generated docs!
    let swagger_ui = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
    let redoc_ui = Redoc::with_url("/redoc", ApiDoc::openapi());
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    let rooms_docs = SwaggerUi::new("/swagger-rooms")
        .url("/api-docs/openapi_rooms.json", ApiDocRooms::openapi());
    let redoc_rooms = Redoc::with_url("/redoc4", ApiDocRooms::openapi());
    let rapidoc_rooms = RapiDoc::new("/api-docs/openapi_rooms.json").path("/rapidoc_rooms");

    let schedule_docs = SwaggerUi::new("/swagger-sched")
        .url("/api-docs/openapi_sched.json", ApiDocSchedule::openapi());
    let redoc_sched = Redoc::with_url("/redoc2", ApiDocSchedule::openapi());
    let rapidoc_sched = RapiDoc::new("/api-docs/openapi_sched.json").path("/rapidoc_sched");

    let speaker_docs = SwaggerUi::new("/swagger-speaker")
        .url("/api-docs/openapi_speaker.json", ApiDocSpeaker::openapi());
    let redoc_speaker = Redoc::with_url("/redoc3", ApiDocSpeaker::openapi());
    let rapidoc_speaker = RapiDoc::new("/api-docs/openapi_speaker.json").path("/rapidoc_speaker");

    let timeslots_docs = SwaggerUi::new("/swagger-timeslots").url(
        "/api-docs/openapi_timeslots.json",
        ApiDocSchedule::openapi(),
    );
    let redoc_timeslot = Redoc::with_url("/redoc5", ApiDocTimeslot::openapi());
    let rapidoc_timeslot =
        RapiDoc::new("/api-docs/openapi_timeslots.json").path("/rapidoc_timeslots");

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/schedules", get(schedule_handler))
        .route("/topics", get(topic_handler))
        .route("/scripts/:path", get(asset_handler))
        .route("/styles/:path", get(asset_handler))
        .nest("/api/v1", apis)
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .merge(schedule_docs)
        .merge(redoc_sched)
        .merge(rapidoc_sched)
        .merge(speaker_docs)
        .merge(redoc_speaker)
        .merge(rapidoc_speaker)
        .merge(rooms_docs)
        .merge(redoc_rooms)
        .merge(rapidoc_rooms)
        .merge(timeslots_docs)
        .merge(redoc_timeslot)
        .merge(rapidoc_timeslot)
        .with_state(topics_db)
        .fallback(handler_404)
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .expose_headers(Any),
        )
        .layer(
            ServiceBuilder::new().layer(trace_layer),
            //.route_service("/favicon.ico", favicon)
        );

    // start up webserver on localhost:3000
    let ip = SocketAddr::new([0, 0, 0, 0].into(), 3000);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    tracing::debug!("serving {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template, Debug)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn index_handler() -> Response {
    IndexTemplate.into_response()
}

#[derive(Debug)]
pub struct Event {
    pub timeslot_id: i32,
    pub title: String,
    pub start_time: String,
    pub end_time: String,
    pub room_id: i32,
    pub topic_id: i32,
    pub speaker_id: i32,
    pub schedule_id: i32,
}

#[derive(Template, Debug)]
#[template(path = "create_schedule.html")]
struct ScheduleTemplate {
    schedule: Option<Schedule>,
    rooms: Option<Vec<Room>>,
    events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
struct CreateScheduleForm {
    num_of_timeslots: i32,
    start_time: Vec<String>,
    end_time: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CreateRoomsForm {
    rooms: Vec<Room>,
}

#[debug_handler]
async fn schedule_handler(State(db_pool): State<Arc<RwLock<UnconfData>>>) -> Response {
    let schedules = {
        let read_lock = db_pool.read().await;
        schedules_get(&read_lock.unconf_db).await.unwrap()
    };
    let rooms = {
        let read_lock = db_pool.read().await;
        match rooms_get(&read_lock.unconf_db).await {
            Ok(None) => None,
            Ok(val) => val,
            _ => None,
        }
    };
    let topics = {
        let read_lock = db_pool.read().await;
        match get_all_topics(&read_lock.unconf_db).await {
            Ok(val) => val,
            _ => vec![],
        }
    };

    let mut events = vec![];
    for schedule in &schedules {
        for timeslot in &schedule.timeslots {
            let event_topic = topics.iter().find(|&topic| topic.id == timeslot.topic_id);
            if event_topic.is_none() {
                continue;
            } else {
                let event = Event {
                    timeslot_id: timeslot.id.unwrap(),
                    title: event_topic.unwrap().title.clone(),
                    start_time: timeslot.start_time.to_string(),
                    end_time: timeslot.end_time.to_string(),
                    room_id: timeslot.room_id.unwrap(),
                    topic_id: timeslot.topic_id.unwrap(),
                    speaker_id: timeslot.speaker_id.unwrap(),
                    schedule_id: schedule.id.unwrap(),
                };

                events.push(event);
            }
        }
    }
    let template = ScheduleTemplate {
        schedule: schedules,
        rooms,
        events,
    };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Template, Debug)]
#[template(path = "topics.html")]
struct TopicsTemplate {
    topics: Vec<TopicAndSpeaker>,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct TopicAndSpeaker {
    #[sqlx(flatten)]
    topic: Topic,
    #[sqlx(flatten)]
    speaker: Speaker,
}

pub async fn combine_topic_and_speaker(
    db_pool: &Pool<Postgres>,
) -> Result<Vec<TopicAndSpeaker>, Box<dyn Error>> {
    let topic_with_speaker: Vec<TopicAndSpeaker> = sqlx::query_as::<Postgres, TopicAndSpeaker>(
        "SELECT t.id, t.speaker_id, t.title, t.content, t.votes, \
        s.id, s.name, s.email, s.phone_number \
        FROM topics t \
        JOIN speakers s ON s.id = t.speaker_id \
        GROUP BY t.id, s.id",
    )
    .fetch_all(db_pool)
    .await?;

    Ok(topic_with_speaker)
}
async fn topic_handler(
    State(db_pool): State<Arc<RwLock<UnconfData>>>,
) -> Response {
    let write_lock = db_pool.write().await;
    let topic_speakers = combine_topic_and_speaker(&write_lock.unconf_db).await;

    match topic_speakers {
        Ok(topic_speakers) => {
            let template = TopicsTemplate {
                topics: topic_speakers,
            };

            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(e) => {
            tracing::debug!(e);
            Html("<h1>Error fetching topics</h1>".to_string()).into_response()
        }
    }
}
