use axum::Router;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace;

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
pub fn configure_middleware(app: Router) -> Router {
    app.layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .expose_headers(Any),
        )
        .layer(
            ServiceBuilder::new()
                .layer(trace::TraceLayer::new_for_http()
                    .make_span_with(trace::DefaultMakeSpan::new())
                    .on_response(trace::DefaultOnResponse::new()))
        )
}