use crate::config::AppState;
use crate::controllers::login_handler::login_page_handler;
use crate::controllers::site_handler::{index_handler, schedule_handler, topic_handler, unconf_timeslots_handler};
use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

/// Creates a new router with the site routes
///
/// This function configures routes for the site:
/// - The index page is served at `/`
/// - The schedule page is served at `/unconf_schedule`
/// - The login page is served at `/login`
/// - The topics page is served at `/topics`
/// - Static assets served from `/scripts` and `/styles`
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// A Router with the site routes
pub fn get_routes(app_state: Arc<RwLock<AppState>>) -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .route("/", get(index_handler))
        .route("/unconf_schedule", get(schedule_handler))
        .route("/login", get(login_page_handler))
        .route("/topics", get(topic_handler))
        .route("/unconf_timeslots", get(unconf_timeslots_handler)).nest_service("/scripts", ServeDir::new("scripts")).nest_service("/styles", ServeDir::new("styles"))
        .with_state(app_state)
}