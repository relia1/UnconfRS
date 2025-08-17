use crate::models::auth_model::{Backend, Permission, User};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_login::AuthSession;
use axum_macros::debug_handler;
use serde_json;
use std::collections::HashSet;

pub type AuthSessionLayer = AuthSession<Backend>;

#[derive(Clone, Debug)]
pub(crate) struct AuthInfo {
    pub(crate) is_authenticated: bool,
    pub(crate) is_staff_or_admin: bool,
    pub(crate) permissions: HashSet<Permission>,
}

pub async fn auth_middleware(auth_session: AuthSessionLayer, mut req: Request, next: Next) -> Response {
    match auth_session.user {
        Some(user) => {
            match auth_session.backend.has_superuser_or_staff_perms(&user).await {
                Ok((is_staff_or_admin, permissions)) => {
                    let auth_info = AuthInfo {
                        is_authenticated: true,
                        is_staff_or_admin,
                        permissions,
                    };

                    req.extensions_mut().insert(auth_info);
                }
                Err(e) => {
                    tracing::error!("Failed to check permissions for user {}: {}", user.id, e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": "false",
                            "message": "Permission check failed"
                        })),
                    ).into_response();
                }
            }
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

pub async fn staff_or_superuser_middleware(
    auth_session: AuthSessionLayer,
    mut req: Request,
    next: Next,
) -> Response {
    match auth_session.user {
        Some(user) => {
            match auth_session.backend.has_superuser_or_staff_perms(&user).await {
                Ok((true, permissions)) => {
                    let auth_info = AuthInfo {
                        is_authenticated: true,
                        is_staff_or_admin: true,
                        permissions,
                    };

                    req.extensions_mut().insert(auth_info);
                }
                Ok((false, _)) => {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(serde_json::json!({
                            "success": "false",
                            "message": "Staff or Admin access required",
                        })),
                    ).into_response();
                }
                Err(e) => {
                    tracing::error!("Failed to check permissions for user {}: {}", user.id, e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": "false",
                            "message": "Permission check failed"
                        })),
                    ).into_response();
                }
            }
        },
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
