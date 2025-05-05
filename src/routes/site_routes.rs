use crate::config::AppState;
use crate::controllers::login_handler::login_page_handler;
use crate::controllers::registration_handler::registration_page_handler;
use crate::controllers::site_handler::{config_handler, index_handler, schedule_handler, session_handler, unconf_timeslots_handler};
use crate::middleware::auth::auth_middleware;
use crate::middleware::unauth::unauth_middleware;
use crate::models::auth_model::Backend;
use axum::middleware::from_fn_with_state;
use axum::{routing::get, Router};
use axum_login::permission_required;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

/// Creates a new router with the site routes
///
/// This function configures routes for the site:
/// - The index page is served at `/`
/// - The schedule page is served at `/unconf_schedule`
/// - The login page is served at `/login`
/// - The sessions page is served at `/sessions`
/// - Static assets served from `/scripts` and `/styles`
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an `Arc` and `RwLock`
///
/// # Returns
/// A Router with the site routes
pub fn get_routes(app_state: Arc<RwLock<AppState>>) -> Router<Arc<RwLock<AppState>>> {
    let site_routes = Router::new()
        .route("/", get(index_handler))
        .route("/unconf_schedule", get(schedule_handler))
        .route("/login", get(login_page_handler))
        .route("/registration", get(registration_page_handler))
        .route("/sessions", get(session_handler))
        .route("/unconf_timeslots", get(unconf_timeslots_handler))
        .route_layer(from_fn_with_state(app_state.clone(), unauth_middleware))
        .nest_service("/scripts", ServeDir::new("scripts"))
        .nest_service("/styles", ServeDir::new("styles"))
        .with_state(app_state.clone());

    let admin_site_routes = Router::new()
        .route("/config", get(config_handler))
        .route_layer(from_fn_with_state(app_state, auth_middleware))
        .route_layer(permission_required!(
            Backend,
            "superuser"
        ));

    site_routes.merge(admin_site_routes)
}
