use crate::config::AppState;
use crate::controllers::site_handler::IndexTemplate;
use crate::middleware::auth::AuthInfo;
use crate::models::auth_model::Permission;
use crate::models::index_model::{add_index_content, IndexContent, IndexMarkdownError};
use crate::types::ApiStatusCode;
use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::{Extension, Form};
use axum_macros::debug_handler;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct IndexMarkdownRequest {
    pub markdown: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/index/markdown",
    request_body = IndexMarkdownRequest,
    responses(
        (status = 200, description = "Index content added or updated", body = [IndexContent]),
        (status = 403, description = "Unauthorized access", body = IndexMarkdownError),
        (status = 500, description = "Unexpected server error", body = IndexMarkdownError),
    )
)]
#[debug_handler]
pub(crate) async fn add_index_markdown(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Extension(auth_info): Extension<AuthInfo>,
    Form(IndexMarkdownRequest { markdown }): Form<IndexMarkdownRequest>,
) -> Response {
    let app_state_lock = app_state.read().await;
    let db_pool = &app_state_lock.unconf_data.read().await.unconf_db;

    match add_index_content(db_pool, auth_info.clone(), &markdown).await {
        Ok(IndexContent { markdown, markdown_converted_to_html }) => {
            let is_authenticated = auth_info.is_authenticated;
            let permissions = auth_info.permissions;
            let is_admin = permissions.contains(&Permission::from("superuser"));

            let template: IndexTemplate = if is_admin {
                IndexTemplate {
                    is_authenticated,
                    permissions,
                    markdown: Some(markdown),
                    markdown_converted_to_html: Some(markdown_converted_to_html),
                }
            } else {
                IndexTemplate {
                    is_authenticated,
                    permissions,
                    markdown: None,
                    markdown_converted_to_html: Some(markdown_converted_to_html),
                }
            };

            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
            }
        }
        Err(e) => {
            let status = if e.to_string().contains("superuser access") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            IndexMarkdownError::response(ApiStatusCode::from(status), e)
        }
    }
}
