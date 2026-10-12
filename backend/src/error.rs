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
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Conflict(&'static str),
    #[error("{message}")]
    Unprocessable { code: &'static str, message: String },
    #[error("something went wrong")]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Extract(#[from] ExtractError),
}

impl ApiError {
    pub fn unprocessable(code: &'static str, message: impl Into<String>) -> Self {
        Self::Unprocessable { code, message: message.into() }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let ApiError::Database(error) = &self {
            tracing::error!(%error, "database error");
        }
        let (status, code) = match &self {
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            ApiError::Conflict(code) => (StatusCode::CONFLICT, *code),
            ApiError::Unprocessable { code, .. } => (StatusCode::UNPROCESSABLE_ENTITY, *code),
            ApiError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            ApiError::Extract(ExtractError::Unsupported(_)) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_file"),
            ApiError::Extract(ExtractError::Network) => (StatusCode::BAD_GATEWAY, "provider_unreachable"),
            ApiError::Extract(ExtractError::Api { .. }) => (StatusCode::BAD_GATEWAY, "provider_error"),
            ApiError::Extract(ExtractError::BadAnswer) => (StatusCode::BAD_GATEWAY, "provider_bad_answer"),
        };
        (status, Json(json!({"error": code, "message": self.to_string()}))).into_response()
    }
}
