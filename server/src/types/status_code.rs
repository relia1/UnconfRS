use axum::http::StatusCode;
use axum::response::{IntoResponse, IntoResponseParts, Response, ResponseParts};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Copy, Clone, Serialize, Deserialize)]

pub struct ApiStatusCode(pub u16);

impl ApiStatusCode {
    pub fn new(status: StatusCode) -> Self {
        Self(status.as_u16())
    }

    pub fn as_status_code(self) -> StatusCode {
        StatusCode::from_u16(self.0).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
impl IntoResponse for ApiStatusCode {
    fn into_response(self) -> Response {
        let status = self.as_status_code();
        (status, ()).into_response()
    }
}

impl IntoResponseParts for ApiStatusCode {
    type Error = (StatusCode, String);

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        Ok(res)
    }
}

impl From<StatusCode> for ApiStatusCode {
    fn from(status: StatusCode) -> Self {
        Self::new(status)
    }
}

impl From<ApiStatusCode> for StatusCode {
    fn from(status: ApiStatusCode) -> Self {
        status.as_status_code()
    }
}

impl Display for ApiStatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
