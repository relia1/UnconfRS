use crate::middleware::auth::AuthInfo;
use crate::models::auth_model::{Backend, Permission};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum_login::{AuthSession, AuthzBackend};
use std::collections::HashSet;

pub type AuthSessionLayer = AuthSession<Backend>;


pub async fn unauth_middleware(auth_session: AuthSessionLayer, mut req: Request, next: Next) -> Response {
    let auth_info = match auth_session.user {
        Some(user) => {
            let permissions =
                auth_session
                    .backend
                    .get_group_permissions(&user)
                    .await
                    .unwrap_or(HashSet::from(Permission { name: String::from("default") }));

            AuthInfo {
                is_authenticated: true,
                is_staff_or_admin: false,
                permissions,
            }
        }
        None => {
            AuthInfo {
                is_authenticated: false,
                is_staff_or_admin: false,
                permissions: HashSet::from(Permission { name: String::from("default") }),
            }
        }
    };

    req.extensions_mut().insert(auth_info);
    next.run(req).await
}
