use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::extractor::ExtractError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error(transparent)]
    Extract(#[from] ExtractError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            ApiError::Extract(ExtractError::Unsupported(_)) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_file"),
            ApiError::Extract(ExtractError::Network) => (StatusCode::BAD_GATEWAY, "provider_unreachable"),
            ApiError::Extract(ExtractError::Api { .. }) => (StatusCode::BAD_GATEWAY, "provider_error"),
            ApiError::Extract(ExtractError::BadAnswer) => (StatusCode::BAD_GATEWAY, "provider_bad_answer"),
        };
        (status, Json(json!({"error": code, "message": self.to_string()}))).into_response()
    }
}
