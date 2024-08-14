mod config;
mod db_config;
mod topics_model;
mod schedule_model;
mod timeslot_model;
mod speakers_handler;
mod speakers_model;
mod topics_handler;
mod schedule_handler;
mod pagination;

use config::*;
use pagination::Pagination;
use schedule_model::{schedules_get, Schedule};
use serde::Deserialize;
use speakers_handler::ApiDocSpeaker;
use timeslot_model::TimeSlot;
// use timeslot_model::ApiDocTimeslot;
use topics_handler::*;
use schedule_handler::*;
use speakers_handler::*;

use topics_model::{paginated_get, Topic};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace,
};
use tracing_subscriber::{fmt, EnvFilter};
extern crate serde_json;
extern crate thiserror;
use axum::{
    extract::{Path, Query, State}, http::StatusCode, response::{Html, IntoResponse, Response}, routing::{delete, get, post, put}, Router
};

// use askama_axum::Template;
use askama_axum::Template;
//use askama::Template;
use utoipa::OpenApi;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{self, sync::RwLock, fs::read_to_string};
extern crate tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

async fn handler_404() -> Response {
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

// handler to load js scripts
async fn script_handler(path: Path<String>) -> String {
    let path = path.to_string();
    let formatted_path = format!("scripts/{}", path);
    read_to_string(formatted_path).await.unwrap()
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
        .route("/speakers", get(speakers))
        .route("/speakers/:id", get(get_speaker))
        .route("/speakers/add", post(post_speaker))
        .route("/speakers/:id", delete(delete_speaker))
        .route("/speakers/:id", put(update_speaker))
        .route("/schedules/:id", get(get_schedule))
        .route("/schedules/:id", put(update_schedule))
        .route("/schedules/add", post(post_schedule))
        .route("/schedules/generate", post(generate));


    // handy openai auto generated docs!
    let swagger_ui = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
    let redoc_ui = Redoc::with_url("/redoc", ApiDoc::openapi());
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    let schedule_docs =
        SwaggerUi::new("/swagger-sched").url("/api-docs/openapi_sched.json", ApiDocSchedule::openapi());
    let redoc_sched = Redoc::with_url("/redoc2", ApiDocSchedule::openapi());
    let rapidoc_sched = RapiDoc::new("/api-docs/openapi_sched.json").path("/rapidoc_sched");

    let speaker_docs =
        SwaggerUi::new("/swagger-speaker").url("/api-docs/openapi_speaker.json", ApiDocSpeaker::openapi());
    let redoc_speaker = Redoc::with_url("/redoc3", ApiDocSpeaker::openapi());
    let rapidoc_speaker = RapiDoc::new("/api-docs/openapi_speaker.json").path("/rapidoc_speaker");

    /*let timeslots_docs =
        SwaggerUi::new("/swagger-timeslots").url("/api-docs/openapi_timeslots.json", ApiDocSchedule::openapi());
    let redoc_timeslot = Redoc::with_url("/redoc3", ApiDocTimeslot::openapi());
    let rapidoc_timeslot = RapiDoc::new("/api-docs/openapi_timeslots.json").path("/rapidoc_timeslots");*/


    let app = Router::new()
        .route("/", get(index_handler))
        .route("/schedules", get(schedule_handler))
        .route("/topics", get(topic_handler))
        .route("/scripts/:path", get(script_handler))
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
        .with_state(topics_db)
        .fallback(handler_404)
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

#[derive(Template, Debug)]
#[template(path = "create_schedule.html")]
struct ScheduleTemplate {
    schedule: Option<Schedule>,
    /*
    * We need to have topic and speaker data returned back as well
    */
}

#[derive(Debug, Deserialize)]
struct CreateScheduleForm {
    num_of_timeslots: i32,
    start_time: Vec<String>,
    end_time: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateSchedule {
    id: i32,
    num_of_timeslots: i32,
    timeslots: Vec<TimeSlot>,
}

async fn schedule_handler(
    State(db_pool): State<Arc<RwLock<UnconfData>>>,
) -> Response {
    let write_lock = db_pool.write().await;
    let schedules = schedules_get(&write_lock.unconf_db).await;
    tracing::trace!("schedules {:?}", schedules);

    match schedules {
        Ok(schedule) => {
            tracing::debug!("{:?}", schedule);
            let template = ScheduleTemplate { schedule };

            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        },
        _ => Html("<h1>Error fetching schedule</h1>".to_string()).into_response(),
    }
}


#[derive(Template, Debug)]
#[template(path = "topics.html")]
struct TopicsTemplate {
    topics: Vec<Topic>,
}

async fn topic_handler(
    State(db_pool): State<Arc<RwLock<UnconfData>>>,
    Query(params): Query<Pagination>,
) -> Response {
    let write_lock = db_pool.write().await;
    let topics = paginated_get(&write_lock.unconf_db, params.page, params.limit).await;

    match topics {
        Ok(topics) => {
            tracing::debug!("{:?}", topics);
            let template = TopicsTemplate { topics };

            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        },
        Err(e) => {
            tracing::debug!(e);
            Html("<h1>Error fetching topics</h1>".to_string()).into_response()
        },
    }
}
