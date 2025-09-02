use crate::middleware::auth::AuthInfo;
use crate::models::auth_model::Permission;
use crate::types::ApiStatusCode;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use pulldown_cmark::Parser;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use sqlx::{Pool, Postgres};
use std::error::Error;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
/// An enumeration of possible errors that can occur.
///
/// # Variants
/// - `ForbiddenAccess` - The `User` does not have access to this endpoint
pub enum IndexMarkdownErr {
    #[error("Attempted access index markdown without having superuser access")]
    ForbiddenAccess,
}

/// Struct representing an error that occurred when working with sessions.
///
/// # Fields
/// - `status` - The HTTP status code associated with the error
/// - `error` - A string describing the specific error that occurred
#[derive(Debug, ToSchema)]
pub struct IndexMarkdownError {
    pub status: ApiStatusCode,
    pub error: String,
}

/// Implements the `Serialize` trait for `IndexMarkdownError`
///
/// This implementation serializes a `IndexMarkdownError` into a JSON object with two properties:
/// `status` and `error`.
impl Serialize for IndexMarkdownError {
    /// Serializes a `SessionError`
    ///
    /// The serialized JSON object will have two properties:
    /// - `status`: A string for the HTTP status code
    /// - `error`: A string describing the error
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status: String = self.status.to_string();
        let mut state = serializer.serialize_struct("IndexMarkdownError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl IndexMarkdownError {
    /// Creates a `Response` instance from a `StatusCode` and `IndexMarkdownErr`.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code.
    /// - `error`: The `IndexMarkdownErr` instance.
    ///
    /// # Returns
    /// `Response` instance with the status code and JSON body containing the error.
    pub fn response(status: ApiStatusCode, error: Box<dyn Error>) -> Response {
        let error = IndexMarkdownError {
            status,
            error: error.to_string(),
        };

        let http_status = StatusCode::from_u16(status.0)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (http_status, Json(error)).into_response()
    }
}

#[derive(Debug, ToSchema, Serialize)]
pub(crate) struct IndexContent {
    pub(crate) markdown: String,
    pub(crate) markdown_converted_to_html: String,
}

pub(crate) async fn add_index_content(
    db_pool: &Pool<Postgres>,
    auth_info: AuthInfo,
    markdown: &str,
) -> Result<IndexContent, Box<dyn Error>> {
    if !auth_info.permissions.contains(&Permission::from("superuser")) {
        return Err(Box::new(IndexMarkdownErr::ForbiddenAccess));
    }

    let mut markdown_converted_to_html = String::new();
    let parser = Parser::new(markdown);
    pulldown_cmark::html::push_html(&mut markdown_converted_to_html, parser);

    let index_content = IndexContent {
        markdown: markdown.to_string(),
        markdown_converted_to_html: markdown_converted_to_html.to_string(),
    };

    sqlx::query!(
        r#"INSERT INTO index_markdown (markdown, markdown_converted_to_html)
        VALUES ($1, $2)
        ON CONFLICT (id) DO UPDATE SET
            markdown = $1,
            markdown_converted_to_html = $2
        "#,
        &index_content.markdown,
        &index_content.markdown_converted_to_html
    )
        .execute(db_pool)
        .await?;

    Ok(index_content)
}
