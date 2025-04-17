extern crate serde_json;
extern crate thiserror;
extern crate tracing;
mod api_docs;
mod config;
mod controllers;
mod db_config;
mod middleware;
mod models;
mod routes;
mod types;

use axum::{http::StatusCode, Router};
use config::*;
use tracing_subscriber::{fmt, EnvFilter};

use crate::controllers::site_handler::handler_404;
use crate::routes::middleware::configure_middleware;
use crate::routes::{api_routes, docs_routes, site_routes};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{self, sync::RwLock};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Setup formatting and environment for trace
    setup_tracing();

    // Connect to database and setup app state
    let app_state = Arc::new(RwLock::new(
        AppState::new().await.unwrap(),
    ));

    // Configure the application router
    let app = configure_app_router(app_state).await;

    // start up webserver on localhost:3000
    let ip = SocketAddr::new([0, 0, 0, 0].into(), 3000);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    tracing::info!(
        "serving {}",
        listener
            .local_addr()
            .unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

/// Set up a tracing subscriber with formatting and filtering
///
/// This functions sets up a tracing subscriber with two layers:
/// - A formatting layer that includes the file and line number
/// - A filter layer that uses the RUST_LOG environment variable
///
/// # Environment Variables
/// - `RUST_LOG` - The log level for the application. If not set, defaults to `info`
///
/// # Panics
/// This function will panic if the tracing subscriber cannot be initialized
fn setup_tracing() {
    let fmt_layer = fmt::layer().with_file(true).with_line_number(true).pretty();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}

/// Configures and returns the application router with all routes and middleware
///
/// This function sets up the application's routing structure by combining all routes found in the
/// routes module and adding middleware to the application.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
///
/// # Returns
/// A configured Router with all routes and middleware
async fn configure_app_router(app_state: Arc<RwLock<AppState>>) -> Router {
    // Get route modules
    let site_routes = site_routes::get_routes(app_state.clone());
    let api_routes = api_routes::get_routes(app_state.clone());
    let docs_routes = docs_routes::get_routes(app_state.clone());

    // Combine routes
    let app = Router::new()
        .merge(site_routes)
        .nest("/api/v1", api_routes)
        .merge(docs_routes)
        .with_state(app_state.clone())
        .fallback(handler_404);

    // Add middleware
    configure_middleware(app, app_state).await
}
