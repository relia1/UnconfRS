use crate::models::auth_model::{Backend, Permission, User};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_login::{AuthSession, AuthzBackend};
use axum_macros::debug_handler;
use serde_json;
use std::collections::HashSet;

pub type AuthSessionLayer = AuthSession<Backend>;

#[derive(Clone, Debug)]
pub(crate) struct AuthInfo {
    pub(crate) is_authenticated: bool,
    pub(crate) permissions: HashSet<Permission>,
}

pub async fn auth_middleware(auth_session: AuthSessionLayer, mut req: Request, next: Next) -> Response {
    match auth_session.user {
        Some(user) => {
            let permissions =
                auth_session
                    .backend
                    .get_group_permissions(&user)
                    .await
                    .unwrap_or(HashSet::from(Permission { name: String::from("default") }));

            let auth_info = AuthInfo {
                is_authenticated: true,
                permissions,
            };

            req.extensions_mut().insert(auth_info);
        }
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                "success": "false",
                "message": "Authentication required"
            })),
            )
                .into_response();
        }
    }

    next.run(req).await
}

#[debug_handler]
pub async fn current_user_handler(auth_session: AuthSessionLayer) -> Response {
    match auth_session.user {
        Some(user) => {
            let user_info = User {
                id: user.id,
                fname: user.fname,
                lname: user.lname,
                email: user.email,
                password: String::new(),
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
