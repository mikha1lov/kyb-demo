use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    compare, db,
    domain::{Comparison, Extracted, Requirement, Typed},
    extractor::{DocumentExtractor, DocumentFile, ExtractError},
};

const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

#[derive(Clone)]
pub struct AppState {
    pub extractor: Arc<dyn DocumentExtractor>,
    pub pool: PgPool,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/documents/analyze", post(analyze))
        .route("/api/documents/compare", post(compare_again))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES + 1024 * 1024))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let database = db::ping(&state.pool).await;
    let status = if database { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (status, Json(json!({"ok": database, "database": database})))
}

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

#[derive(Serialize)]
struct RequirementResult {
    requirement: Requirement,
    comparison: Comparison,
}

fn evaluate(requirements: &[Requirement], typed: &Typed, extracted: &Extracted) -> Vec<RequirementResult> {
    let today = Utc::now().date_naive();
    requirements
        .iter()
        .map(|&requirement| RequirementResult {
            requirement,
            comparison: compare::compare(requirement, typed, extracted, today),
        })
        .collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompareRequest {
    requirements: Vec<Requirement>,
    typed: Typed,
    extracted: Extracted,
}

#[derive(Serialize)]
struct CompareResponse {
    results: Vec<RequirementResult>,
}

async fn compare_again(Json(request): Json<CompareRequest>) -> Json<CompareResponse> {
    Json(CompareResponse { results: evaluate(&request.requirements, &request.typed, &request.extracted) })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyzeResponse {
    extractor: String,
    extracted: Extracted,
    results: Vec<RequirementResult>,
}

/// multipart/form-data:
///   file         the document (pdf, png, jpeg, webp, up to 10 MB)
///   requirement  formation | source_of_funds | proof_of_address (repeat the field for several)
///   typed        JSON with what the user typed: {"legalName": "...", "registrationNumber": "...", "address": "..."}
async fn analyze(State(state): State<AppState>, mut form: Multipart) -> Result<Json<AnalyzeResponse>, ApiError> {
    let mut file: Option<DocumentFile> = None;
    let mut requirements: Vec<Requirement> = Vec::new();
    let mut typed = Typed::default();

    while let Some(field) = form.next_field().await.map_err(|e| ApiError::BadRequest(e.to_string()))? {
        match field.name().unwrap_or_default() {
            "file" => {
                let filename = field.file_name().unwrap_or("document").to_string();
                let mime = field.content_type().unwrap_or("application/octet-stream").to_string();
                let bytes = field.bytes().await.map_err(|e| ApiError::BadRequest(e.to_string()))?.to_vec();
                if bytes.len() > MAX_UPLOAD_BYTES {
                    return Err(ApiError::BadRequest("file is larger than 10 MB".into()));
                }
                file = Some(DocumentFile { filename, mime, bytes });
            }
            "requirement" => {
                let text = field.text().await.map_err(|e| ApiError::BadRequest(e.to_string()))?;
                requirements.push(
                    serde_json::from_value(json!(text.trim()))
                        .map_err(|_| ApiError::BadRequest("unknown requirement".into()))?,
                );
            }
            "typed" => {
                let text = field.text().await.map_err(|e| ApiError::BadRequest(e.to_string()))?;
                typed = serde_json::from_str(&text).map_err(|_| ApiError::BadRequest("typed is not valid JSON".into()))?;
            }
            _ => {}
        }
    }

    let file = file.ok_or_else(|| ApiError::BadRequest("file is required".into()))?;
    if requirements.is_empty() {
        return Err(ApiError::BadRequest("requirement is required".into()));
    }

    // Log sizes and outcome only, never document contents or extracted values.
    tracing::info!(bytes = file.bytes.len(), mime = %file.mime, ?requirements, "analyzing document");

    let extracted = state.extractor.extract(file).await?;
    let results = evaluate(&requirements, &typed, &extracted);
    tracing::info!(requirements = results.len(), "document analysed");

    Ok(Json(AnalyzeResponse { extractor: state.extractor.label(), extracted, results }))
}
