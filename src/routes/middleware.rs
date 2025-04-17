use crate::config::AppState;
use axum::Router;
use axum_login::{
    tower_sessions::{Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_cookies::{cookie::time::Duration, CookieManagerLayer};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace,
};
use tower_sessions_sqlx_store::PostgresStore;


/// Configures middleware for the application
///
/// This function configures middleware for the application. It adds compression, CORS, and tracing
/// middleware to the application.
///
/// # Parameters
/// - `app` - The application to configure the middleware for
///
/// # Returns
/// The application with the configured middleware
pub async fn configure_middleware(app: Router, app_state: Arc<RwLock<AppState>>) -> Router {
    let read_lock = app_state.read().await;
    let session_store = PostgresStore::new(read_lock.unconf_data.read().await.unconf_db.clone());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));
    let auth_layer = AuthManagerLayerBuilder::new(read_lock.auth_backend.clone(), session_layer).build();

    app.layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .expose_headers(Any),
        )
        .layer(
            ServiceBuilder::new().layer(
                trace::TraceLayer::new_for_http()
                    .make_span_with(trace::DefaultMakeSpan::new())
                    .on_response(trace::DefaultOnResponse::new()),
            ),
        )
        .layer(CookieManagerLayer::new())
        .layer(auth_layer)
}
