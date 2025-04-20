use crate::api_docs::ApiDoc;
use crate::config::AppState;
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

/// Creates a new router with the documentation routes
///
/// This function configures routes for the Swagger UI, `ReDoc`, and `RapiDoc`:
/// - Swagger UI is served at `/swagger-ui`
/// - `ReDoc` is served at `/redoc`
/// - `RapiDoc` is served at `/rapidoc`
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an `Arc` and `RwLock`
///
/// # Returns
/// A Router with the documentation routes
pub fn get_routes(app_state: Arc<RwLock<AppState>>) -> Router<Arc<RwLock<AppState>>> {
    let swagger_ui = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
    let redoc = Redoc::with_url("/redoc", ApiDoc::openapi());
    let rapidoc = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    Router::new()
        .merge(swagger_ui)
        .merge(redoc)
        .merge(rapidoc)
        .with_state(app_state)
}
