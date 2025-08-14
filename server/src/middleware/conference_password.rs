use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use tower_sessions::Session;

/// Configures middleware conference access
///
/// This function ensures only those who have logged in to the general unconference login can
/// access the site
///
/// # Parameters
/// - `session` - The site session
/// - `req` - The request object
/// - `next` - The rest of the middleware stack
///
/// # Returns
/// A `Response`
pub async fn conference_password_middleware(session: Session, req: Request, next: Next) -> Response {
    match session.get::<bool>("conference_authenticated").await {
        Ok(Some(true)) => {
            next.run(req).await
        }
        _ => {
            Redirect::to("/conference_login").into_response()
        }
    }
}