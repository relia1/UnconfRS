use crate::config::AppState;
use crate::controllers::registration_handler::registration_handler;
use crate::controllers::{
    login_handler::{login_handler, logout_handler},
    room_handler::{delete_room, post_rooms, rooms},
    schedule_handler::{clear, generate},
    session_voting_handler::{add_vote_for_session, subtract_vote_for_session},
    sessions_handler::{
        delete_session, get_session, post_session, sessions, update_session,
    },
    timeslot_handler::{add_timeslots, swap_timeslots, update_timeslot},
};
use crate::middleware::auth::{auth_middleware, current_user_handler};
use crate::models::auth_model::Backend;
use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Router,
};
use axum_login::permission_required;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Returns a router with all the routes for the API
///
/// This function returns a router with all the routes for the API. It includes routes for sessions,
/// rooms, schedules, timeslots, and authentication.
///
/// # Parameters
/// - `app_state` - The shared application state wrapped in an `Arc` and `RwLock`
///
/// # Returns
/// A router with all the routes for the API
pub fn get_routes(app_state: &Arc<RwLock<AppState>>) -> Router<Arc<RwLock<AppState>>> {
    let public_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/registration", post(registration_handler))
        .route("/sessions", get(sessions))
        .route("/sessions/{id}", get(get_session))
        .route("/rooms", get(rooms));

    let auth_routes = Router::new()
        .route("/logout", post(logout_handler))
        .route("/current_user", get(current_user_handler))
        .route("/sessions/add", post(post_session))
        .route("/sessions/{id}", delete(delete_session))
        .route("/sessions/{id}", put(update_session))
        .route("/sessions/{id}/increment", put(add_vote_for_session))
        .route("/sessions/{id}/decrement", put(subtract_vote_for_session))
        .route_layer(from_fn_with_state(app_state.clone(), auth_middleware));

    let admin_routes = Router::new()
        .route("/rooms/add", post(post_rooms))
        .route("/rooms/{id}", delete(delete_room))
        .route("/schedules/generate", post(generate))
        .route("/schedules/clear", post(clear))
        .route("/timeslots/{id}", put(update_timeslot))
        .route("/timeslots/add", post(add_timeslots))
        .route("/timeslots/swap", put(swap_timeslots))
        .route_layer(from_fn_with_state(app_state.clone(), auth_middleware))
        .route_layer(permission_required!(
            Backend,
            "superuser"
        ));

    public_routes.merge(auth_routes.merge(admin_routes))
}
