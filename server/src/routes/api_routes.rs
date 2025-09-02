use crate::config::AppState;
use crate::controllers::index_handler::add_index_markdown;
use crate::controllers::registration_handler::{registration_handler, staff_registers_user_handler};
use crate::controllers::schedule_handler::{add_session_to_schedule, remove_session_from_schedule};
use crate::controllers::sessions_handler::post_session_for_user;
use crate::controllers::tags_handler::{create_tag, delete_tag, update_tag};
use crate::controllers::{login_handler::{login_handler, logout_handler}, room_handler::{delete_room, post_rooms, rooms}, schedule_handler::{clear, generate}, session_tags_handler::{add_tag_for_session, remove_tag_for_session, update_tag_for_session}, session_voting_handler::{add_vote_for_session, subtract_vote_for_session}, sessions_handler::{
    delete_session, get_session, post_session, sessions, update_session,
}, timeslot_handler::{add_timeslots, swap_timeslots, update_timeslot}};
use crate::middleware::auth::{auth_middleware, current_user_handler};
use crate::middleware::unauth::unauth_middleware;
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
        .route("/rooms", get(rooms))
        .route_layer(from_fn_with_state(app_state.clone(), unauth_middleware));

    let auth_routes = Router::new()
        .route("/logout", post(logout_handler))
        .route("/current_user", get(current_user_handler))
        .route("/sessions/add", post(post_session))
        .route("/sessions/{id}", delete(delete_session))
        .route("/sessions/{id}", put(update_session))
        .route("/sessions/{id}/increment", put(add_vote_for_session))
        .route("/sessions/{id}/decrement", put(subtract_vote_for_session))
        .route("/sessions/{id}/tags", post(add_tag_for_session).put(update_tag_for_session).delete(remove_tag_for_session))
        .route_layer(from_fn_with_state(app_state.clone(), auth_middleware));

    let staff_or_admin_routes = Router::new()
        .route("/sessions/add_for_user", post(post_session_for_user))
        .route("/registration_on_user_behalf", post(staff_registers_user_handler))
        .route_layer(from_fn_with_state(app_state.clone(), auth_middleware));

    let admin_routes = Router::new()
        .route("/rooms/add", post(post_rooms))
        .route("/rooms/{id}", delete(delete_room))
        .route("/schedules/generate", post(generate))
        .route("/schedules/clear", post(clear))
        .route("/schedules/add_session", post(add_session_to_schedule))
        .route("/schedules/remove_session", post(remove_session_from_schedule))
        .route("/timeslots/{id}", put(update_timeslot))
        .route("/timeslots/add", post(add_timeslots))
        .route("/timeslots/swap", put(swap_timeslots))
        .route("/tags", post(create_tag))
        .route("/tags/{id}", put(update_tag))
        .route("/tags/{id}", delete(delete_tag))
        .route("/index/markdown", post(add_index_markdown))
        .route_layer(from_fn_with_state(app_state.clone(), auth_middleware))
        .route_layer(permission_required!(
            Backend,
            "superuser"
        ));

    public_routes
        .merge(auth_routes)
        .merge(staff_or_admin_routes)
        .merge(admin_routes)
}
