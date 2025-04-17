use crate::config::AppState;
use crate::controllers::{
    login_handler::{login_handler, logout_handler},
    room_handler::{delete_room, post_rooms, rooms},
    schedule_handler::{clear, generate, get_schedule, post_schedule, schedules},
    speakers_handler::{delete_speaker, get_speaker, post_speaker, speakers, update_speaker},
    timeslot_handler::{add_timeslots, swap_timeslots, update_timeslot},
    topics_handler::{
        add_vote_for_topic, delete_topic, get_topic, post_topic, subtract_vote_for_topic, topics,
        update_topic,
    },
};
use crate::middleware::auth::{auth_middleware, current_user_handler};
use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Router,
};
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
    let public_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/topics", get(topics))
        .route("/topics/:id", get(get_topic))
        .route("/rooms", get(rooms))
        .route("/speakers", get(speakers))
        .route("/speakers/:id", get(get_speaker))
        .route("/schedules", get(schedules))
        .route("/schedules/:id", get(get_schedule));

    let auth_routes = Router::new()
        .route("/logout", post(logout_handler))
        .route("/current_user", get(current_user_handler))
        .route("/topics/add", post(post_topic))
        .route("/topics/:id", delete(delete_topic))
        .route("/topics/:id", put(update_topic))
        .route("/topics/:id/increment", put(add_vote_for_topic))
        .route("/topics/:id/decrement", put(subtract_vote_for_topic))
        .route("/rooms/add", post(post_rooms))
        .route("/rooms/:id", delete(delete_room))
        .route("/speakers/add", post(post_speaker))
        .route("/speakers/:id", delete(delete_speaker))
        .route("/speakers/:id", put(update_speaker))
        .route("/schedules/add", post(post_schedule))
        .route("/schedules/generate", post(generate))
        .route("/schedules/clear", post(clear))
        .route("/timeslots/:id", put(update_timeslot))
        .route("/timeslots/add", post(add_timeslots))
        .route("/timeslots/swap", put(swap_timeslots))
        .route_layer(from_fn_with_state(app_state.clone(), auth_middleware));

    public_routes.merge(auth_routes)
}
