use crate::config::AppState;
use crate::models::room_model::{rooms_get, Room};
use crate::models::schedule_model::{schedules_get, Schedule};
use crate::models::speakers_model::Speaker;
use crate::models::timeslot_model::{timeslot_get, ExistingTimeslot, TimeslotAssignment};
use crate::models::topics_model::{get_all_topics, Topic};
use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

#[debug_handler]
/// Fall back handler
///
/// This function is a handler for requests that do not match any other route.
///
/// # Returns
/// `Response` with a status code of 404 Not Found.
pub async fn handler_404() -> Response {
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

#[derive(Template, Debug)]
#[template(path = "index.html")]
/// Index template
struct IndexTemplate;

#[debug_handler]
/// Index handler
///
/// This function renders the index page.
///
/// # Returns
/// `Response` with the rendered HTML page or an error status code.
///
/// # Errors
/// If the template fails to render, an internal server error status code is returned.
pub async fn index_handler() -> Response {
    IndexTemplate.into_response()
}

#[derive(Debug, Serialize)]
/// Event struct
///
/// This struct represents the parameters of an event.
///
/// # Fields
/// - `timeslot_id` - The ID of the timeslot
/// - `title` - The title of the event
/// - `start_time` - The start time of the event
/// - `end_time` - The end time of the event
/// - `room_id` - The ID of the room
/// - `topic_id` - The ID of the topic
/// - `speaker_id` - The ID of the speaker
/// - `schedule_id` - The ID of the schedule
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

#[derive(Template, Debug, Serialize)]
#[template(path = "create_schedule.html")]
pub(crate) struct ScheduleTemplate {
    pub(crate) schedule: Option<Schedule>,
    pub(crate) rooms: Option<Vec<Room>>,
    pub(crate) events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
/// Create schedule form
///
/// This struct represents the parameters of the create schedule form.
///
/// # Fields
/// - `num_of_timeslots` - The number of timeslots
/// - `start_time` - The start time
/// - `end_time` - The end time
pub(crate) struct CreateScheduleForm {
    pub(crate) num_of_timeslots: i32,
    pub(crate) start_time: Vec<String>,
    pub(crate) end_time: Vec<String>,
}

#[debug_handler]
/// Schedule handler
///
/// This function renders the schedule page.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with the rendered HTML page or an error status code.
///
/// # Errors
/// If the template fails to render, an internal server error status code is returned.
pub async fn schedule_handler(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;

    let result: Result<String, Response> = async {
        let schedule = schedules_get(read_lock)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

        let rooms = rooms_get(read_lock).await.unwrap_or(None);
        let topics = get_all_topics(read_lock).await.unwrap_or_default();
        let timeslots = timeslot_get(read_lock)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
        let assignments = sqlx::query_as::<_, TimeslotAssignment>(
            "SELECT * FROM timeslot_assignments"
        )
            .fetch_all(read_lock)
            .await
            .unwrap_or_default();

        let events = if let Some(schedule) = &schedule {
            let schedule_id = schedule.id.ok_or(StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
            timeslots.iter().flat_map(|timeslot| {
                let timeslot_id = timeslot.id;

                assignments.iter()
                           .filter(|assignment| assignment.time_slot_id == timeslot_id)
                           .filter_map(|filtered_assignment| {
                               let event_topic = topics.iter().find(|&topic| topic.id == Some(filtered_assignment.topic_id))?;

                               Some(Event {
                                   timeslot_id,
                                   title: event_topic.title.clone(),
                                   start_time: timeslot.start_time.to_string(),
                                   end_time: timeslot.end_time.to_string(),
                                   room_id: filtered_assignment.room_id,
                                   topic_id: filtered_assignment.topic_id,
                                   speaker_id: filtered_assignment.speaker_id,
                                   schedule_id,
                               })
                           })
                           .collect::<Vec<_>>()
            }).collect()
        } else {
            vec![]
        };

        let template = ScheduleTemplate {
            schedule,
            rooms,
            events,
        };

        template.render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
        .await;

    match result {
        Ok(html) => Html(html).into_response(),
        Err(response) => response,
    }
}
#[derive(Template, Debug)]
#[template(path = "topics.html")]
/// Topics template
///
/// This struct represents the parameters passed to the client for rendering the topics page.
///
/// # Fields
/// - `topics` - A vector containing the topics and speakers
struct TopicsTemplate {
    topics: Vec<TopicAndSpeaker>,
}

#[derive(Debug, Deserialize, FromRow)]
/// Topic and speaker struct
///
/// This struct represents the pairing of a topic and a speaker.
///
/// # Fields
/// - `topic` - The session topic
/// - `speaker` - The speaker for the topic
pub struct TopicAndSpeaker {
    #[sqlx(flatten)]
    topic: Topic,
    #[sqlx(flatten)]
    speaker: Speaker,
}

/// Combined topic and speaker query
///
/// This function queries the database for a combination of topics and speakers.
///
/// # Parameters
/// - `db_pool` - The database connection pool
///
/// # Returns
/// A vector containing the topics and speakers or an error if the query fails.
///
/// # Errors
/// An error is returned if the query fails.
pub async fn combine_topic_and_speaker(
    db_pool: &Pool<Postgres>,
) -> Result<Vec<TopicAndSpeaker>, Box<dyn Error>> {
    let topic_with_speaker: Vec<TopicAndSpeaker> = sqlx::query_as::<Postgres, TopicAndSpeaker>(
        "SELECT t.id, t.speaker_id, t.title, t.content, t.votes, \
        s.id, s.name, s.email, s.phone_number \
        FROM topics t \
        JOIN speakers s ON s.id = t.speaker_id \
        GROUP BY t.id, s.id",
    ).fetch_all(db_pool).await?;

    Ok(topic_with_speaker)
}

#[debug_handler]
/// Topic handler
///
/// This function renders the topics page.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// `Response` with the rendered HTML page or an error status code.
///
/// # Errors
/// If the template fails to render, an internal server error status code is returned.
pub async fn topic_handler(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;
    let topic_speakers = combine_topic_and_speaker(write_lock).await;

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


#[derive(Template, Clone, Deserialize, FromRow)]
#[template(path = "unconf_timeslots.html")]
struct UnconfTimeslotsTemplate {
    existing_timeslots: Vec<ExistingTimeslot>,
}

#[debug_handler]
pub async fn unconf_timeslots_handler(State(app_state): State<Arc<RwLock<AppState>>>) -> Response {
    let app_state_lock = app_state.read().await;
    let read_lock = &app_state_lock.unconf_data.read().await.unconf_db;

    let timeslots = match timeslot_get(read_lock).await {
        Ok(timeslots) => timeslots,
        Err(e) => {
            tracing::error!("Error getting timeslots: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        },
    };

    tracing::trace!("Timeslots: {:?}", timeslots);

    let template = UnconfTimeslotsTemplate {
        existing_timeslots: timeslots,
    };


    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Error rendering template: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        },
    }
}