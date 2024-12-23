use crate::config::AppState;
use crate::controllers::room_handler::{delete_room, post_rooms, rooms};
use crate::controllers::schedule_handler::{
    clear, generate, get_schedule, post_schedule, schedules, update_schedule,
};
use crate::controllers::speakers_handler::{
    delete_speaker, get_speaker, post_speaker, speakers, update_speaker,
};
use crate::controllers::timeslot_handler::{add_timeslots, update_timeslot};
use crate::controllers::topics_handler::{
    add_vote_for_topic, delete_topic, get_topic, post_topic, subtract_vote_for_topic, topics,
    update_topic,
};
use crate::middleware::auth::auth_middleware;
use crate::models::admin_model::admin_login;
use axum::handler::Handler;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, post, put};
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Returns a router with all the routes for the API
///
/// This function returns a router with all the routes for the API. It includes routes for topics,
/// rooms, speakers, schedules, and authentication.
///
/// # Parameters
/// - `app_state` - The shared application state wrapped in an Arc and RwLock
///
/// # Returns
/// A router with all the routes for the API
pub fn get_routes(app_state: Arc<RwLock<AppState>>) -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .route("/admin_login", post(admin_login))
        .route("/topics", get(topics))
        .route("/topics/:id", get(get_topic))
        .route("/topics/add", post(post_topic))
        .route("/topics/:id", delete(delete_topic))
        .route("/topics/:id", put(update_topic))
        .route("/topics/:id/increment", put(add_vote_for_topic))
        .route("/topics/:id/decrement", put(subtract_vote_for_topic))
        // Room routes
        .route("/rooms", get(rooms))
        .route("/rooms/add", post(post_rooms))
        .route("/rooms/:id", delete(delete_room))
        // Speaker routes
        .route("/speakers", get(speakers))
        .route("/speakers/:id", get(get_speaker))
        .route("/speakers/add", post(post_speaker))
        .route("/speakers/:id", delete(delete_speaker))
        .route("/speakers/:id", put(update_speaker))
        // Schedule routes with authentication
        .route("/schedules", get(schedules))
        .route("/schedules/:id", get(get_schedule))
        .route("/schedules/:id", put(update_schedule))
        .route("/schedules/add", post(post_schedule))
        .route(
            "/schedules/generate",
            post(generate.layer(from_fn_with_state(app_state.clone(), auth_middleware))),
        )
        .route(
            "/schedules/clear",
            post(clear.layer(from_fn_with_state(app_state.clone(), auth_middleware))),
        )
        .route(
            "/timeslots/:id",
            put(update_timeslot.layer(from_fn_with_state(app_state.clone(), auth_middleware))),
        )
        .route(
            "/timeslots/add",
            post(add_timeslots.layer(from_fn_with_state(app_state.clone(), auth_middleware))),
        )
}
