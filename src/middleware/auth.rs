use crate::models::auth_model::{Backend, User};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_login::AuthSession;
use axum_macros::debug_handler;

pub type AuthSessionLayer = AuthSession<Backend>;

pub async fn auth_middleware(auth_session: AuthSessionLayer, req: Request, next: Next) -> Response {
    if auth_session.user.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": "false",
                "message": "Authentication required"
            })),
        )
            .into_response();
    }

    next.run(req).await
}

#[debug_handler]
pub async fn current_user_handler(auth_session: AuthSessionLayer) -> Response {
    match auth_session.user {
        Some(user) => {
            let user_info = User {
                id: user.id,
                username: user.username,
                password: "".to_string(),
            };
            (StatusCode::OK, Json(user_info)).into_response()
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"message": "Unauthorized"})),
        )
            .into_response(),
    }
}
