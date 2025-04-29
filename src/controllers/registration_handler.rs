use crate::middleware::auth::AuthSessionLayer;
use crate::models::auth_model::{RegistrationRequest, RegistrationResponse};
use askama::Template;
use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Html, response::Response, Json};
use axum_macros::debug_handler;

#[derive(Template, Debug)]
#[template(path = "registration.html")]
struct RegistrationTemplate {
    is_authenticated: bool,
}

#[debug_handler]
/// Login page handler
///
/// This function renders the login page.
///
/// # Returns
/// `Response` with the rendered HTML page or an error status code.
///
/// # Errors
/// If the template fails to render, an internal server error status code is returned.
pub async fn registration_page_handler() -> Response {
    let template = RegistrationTemplate { is_authenticated: false };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[debug_handler]
pub async fn registration_handler(
    auth_session: AuthSessionLayer,
    Json(user_info): Json<RegistrationRequest>,
) -> impl IntoResponse {
    tracing::trace!("user_info: {:?}", user_info.fname);
    match auth_session.backend.register(user_info).await {
        Ok(()) => {
            (
                StatusCode::CREATED,
                Json(RegistrationResponse {
                    success: true,
                    message: "User created".to_string(),
                }),
            )
        }
        Err(_) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegistrationResponse {
                    success: false,
                    message: "Internal server error".to_string(),
                }),
            )
        }
    }
}
