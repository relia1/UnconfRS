use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::response::Html;
use axum_macros::debug_handler;

#[derive(Template, Debug)]
#[template(path = "admin_login.html")]
struct AdminTemplate;

#[debug_handler]
/// Admin handler
/// 
/// This function renders the admin page.
/// 
/// # Returns
/// `Response` with the rendered HTML page or an error status code.
/// 
/// # Errors
/// If the template fails to render, an internal server error status code is returned.
pub async fn admin_handler() -> Response {
    let template = AdminTemplate {};
    match template.render() { 
        Ok(html) => Html(html).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}