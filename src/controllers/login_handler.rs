use crate::middleware::auth::AuthSessionLayer;
use crate::models::auth_model::{Credentials, LoginRequest, LoginResponse};
use askama::Template;
use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Html, response::Response, Json};
use axum_macros::debug_handler;

#[derive(Template, Debug)]
#[template(path = "login.html")]
struct LoginTemplate;

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
pub async fn login_page_handler() -> Response {
    let template = LoginTemplate {};
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[debug_handler]
pub async fn login_handler(
    mut auth_session: AuthSessionLayer,
    Json(credentials): Json<LoginRequest>,
) -> impl IntoResponse {
    tracing::trace!("Logging in");
    let creds = Credentials {
        username: credentials.username,
        password: credentials.password,
    };

    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse {
                    success: false,
                    message: "Invalid credentials".to_string(),
                }),
            )
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse {
                    success: false,
                    message: "Internal server error".to_string(),
                }),
            )
        }
    };

    if auth_session.login(&user).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LoginResponse {
                success: false,
                message: "Failed to create session".to_string(),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(LoginResponse {
            success: true,
            message: "Login successful".to_string(),
        }),
    )
}

pub async fn logout_handler(mut auth_session: AuthSessionLayer) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(_) => (
            StatusCode::OK,
            Json(LoginResponse {
                success: true,
                message: "Logout successful".to_string(),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LoginResponse {
                success: false,
                message: "Failed to logout".to_string(),
            }),
        ),
    }
}
