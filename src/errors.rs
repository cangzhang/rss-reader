use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
pub enum CustomError {
    BadRequest,
    InternalServerError,
    InvalidCredentials,
}

impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::BadRequest => (StatusCode::BAD_REQUEST, "Bad Request"),
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid Credentials"),
        };
        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
