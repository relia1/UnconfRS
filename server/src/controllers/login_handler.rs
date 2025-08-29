use crate::config::AppState;
use crate::middleware::auth::{AuthInfo, AuthSessionLayer};
use crate::models::auth_model::{Credentials, LoginRequest, LoginResponse, Permission};
use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::{http::StatusCode, response::Html, response::Response, Extension, Form, Json};
use axum_macros::debug_handler;
use serde::Deserialize;
use sqlx::FromRow;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_sessions::Session;

#[derive(Template, Debug)]
#[template(path = "login.html")]
struct LoginTemplate {
    is_authenticated: bool,
    permissions: HashSet<Permission>,
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
pub(crate) async fn login_page_handler(Extension(auth_info): Extension<AuthInfo>) -> Response {
    let template = LoginTemplate { is_authenticated: auth_info.is_authenticated, permissions: auth_info.permissions };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Login handler
///
/// This function handles a user attempting to authenticate.
///
/// # Returns
/// `impl IntoResponse` with the rendered HTML page or an error status code.
///
/// # Errors
/// If the login fails an UNAUTHORIZED error is returned
/// Otherwise INTERNAL_SERVER_ERROR
#[debug_handler]
pub(crate) async fn login_handler(
    mut auth_session: AuthSessionLayer,
    Json(credentials): Json<LoginRequest>,
) -> impl IntoResponse {
    let creds = Credentials {
        email: credentials.email,
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


#[derive(Template)]
#[template(path = "unconference_password.html")]
pub struct UnconferencePasswordTemplate {
    pub error_message: Option<String>,
}

#[derive(Deserialize, FromRow)]
pub struct UnconferencePasswordForm {
    pub password: String,
}

#[derive(FromRow)]
pub struct UnconferencePassword {
    pub password: String,
}

/// Unconference login page handler
///
/// This function renders the unconference login page.
///
/// # Returns
/// `impl IntoResponse` with the rendered HTML page or an error status code.
///
/// # Errors
/// If the template fails to render, an internal server error status code is returned.
#[debug_handler]
pub async fn unconference_password_page_handler() -> impl IntoResponse {
    let template = UnconferencePasswordTemplate {
        error_message: None,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Error rendering template: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Unconference login handler
///
/// This function handles a user attempting to authenticate to the unconference login.
///
/// # Returns
/// `Response` with the rendered HTML page (includes error message if there was an error).
///
/// # Errors
/// Failure to query, bcrypt, or render the template lead to INTERNAL_SERVER_ERROR
#[debug_handler]
pub async fn unconference_password_submit_handler(
    session: Session,
    State(app_state): State<Arc<RwLock<AppState>>>,
    Form(form): Form<UnconferencePasswordForm>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let write_lock = &app_state_lock.unconf_data.read().await.unconf_db;

    let unconference_password = sqlx::query_as!(
        UnconferencePassword,
        r"SELECT password FROM conference_password limit 1",
    )
        .fetch_optional(write_lock)
        .await;

    match unconference_password {
        Ok(Some(UnconferencePassword { password })) => {
            if bcrypt::verify(&form.password, &password).is_ok_and(|x| x) {
                if (session.insert("unconference_authenticated", true).await).is_err() {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }

                Redirect::to("/").into_response()
            } else {
                let template = UnconferencePasswordTemplate {
                    error_message: Some("Invalid password".to_string()),
                };

                match template.render() {
                    Ok(html) => (StatusCode::UNAUTHORIZED, Html(html)).into_response(),
                    Err(e) => {
                        tracing::error!("Error rendering template: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            }
        },
        Ok(None) => {
            let template = UnconferencePasswordTemplate {
                error_message: Some("Unconference password not configured".to_string()),
            };

            match template.render() {
                Ok(html) => (StatusCode::SERVICE_UNAVAILABLE, Html(html)).into_response(),
                Err(e) => {
                    tracing::error!("Error rendering template: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        },
        Err(_) => {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
