use std::sync::Arc;
use askama_axum::IntoResponse;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use serde::Deserialize;
use serde_json::from_str;
use sqlx::FromRow;
use tokio::sync::RwLock;
use crate::config::AppState;

#[derive(Debug, Deserialize, FromRow)]
/// User struct
///
/// This struct represents a user with a username and password.
///
/// # Fields
/// - `username` - The username of the user
/// - `password` - The hashed/salted password of the user
pub struct User {
    password: String,
}

/// Admin login handler
///
/// This function handles the login of an admin user. It checks the username and password against
/// the database and if they match, it returns an authorized response with a JWT token in a cookie.
///
/// # Parameters
/// - `app_state` - Thread-safe shared state wrapped in an Arc and RwLock
/// - `admin_form` - The JSON body containing the username and password of the admin user
///
/// # Returns
/// If the username and password match, an authorized response with a JWT token in a cookie is
/// returned otherwise an unauthorized response is returned.
///
/// # Errors
/// If the username and password do not match, a response with a status code of 401 Unauthorized is
/// returned.
///
/// # Panics
/// This method panics if capacity exceeds max HeaderMap capacity (headers.insert() call).
pub async fn admin_login(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(admin_form): Json<User>,
) -> impl IntoResponse {
    let app_state_lock = app_state.read().await;
    let jwt_token = app_state_lock.jwt_secret.read().await.clone();
    let db_pool = &app_state_lock.unconf_data.read().await.unconf_db;
    let db_user_result: Result<User, _>  = sqlx::query_as("SELECT * FROM users WHERE username = \
    $1;")
        .bind("admin")
        .fetch_one(db_pool)
        .await;

    let db_user = match db_user_result {
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, HeaderMap::new(), "Unauthorized");
        },
        Ok(user) => {
            user
        }
    };

    match bcrypt::verify(&admin_form.password, &db_user.password) {
        Ok(true) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::SET_COOKIE,
                HeaderValue::from_str(&format!(
                    "token={}; HttpOnly; Secure; SameSite=Strict; Path=/",
                    jwt_token
                )).unwrap_or(HeaderValue::from_static("")),
            );
            (
                StatusCode::OK,
                headers,
                "Authorized"
            )
        },
        _ => {
            (StatusCode::UNAUTHORIZED, HeaderMap::new(), "Unauthorized")
        }
    }
}