use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::{
    applications,
    compare::{self, RequirementResult},
    db,
    domain::{Extracted, Requirement, Typed},
    error::ApiError,
    extractor::{DocumentExtractor, DocumentFile},
};

pub const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

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
        .merge(applications::routes())
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES + 1024 * 1024))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let database = db::ping(&state.pool).await;
    let status = if database { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (status, Json(json!({"ok": database, "database": database})))
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

fn evaluate_alone(requirements: &[Requirement], typed: &Typed, extracted: &Extracted) -> Vec<RequirementResult> {
    let kind = compare::number_kind(typed.registration_number.as_deref(), &[extracted]);
    compare::evaluate(requirements, typed, extracted, Utc::now().date_naive(), kind)
}

async fn compare_again(Json(request): Json<CompareRequest>) -> Json<CompareResponse> {
    Json(CompareResponse { results: evaluate_alone(&request.requirements, &request.typed, &request.extracted) })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyzeResponse {
    extractor: String,
    extracted: Extracted,
    results: Vec<RequirementResult>,
}

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

    tracing::info!(bytes = file.bytes.len(), mime = %file.mime, ?requirements, "analyzing document");

    let extracted = state.extractor.extract(file).await?;
    let results = evaluate_alone(&requirements, &typed, &extracted);
    tracing::info!(requirements = results.len(), "document analysed");

    Ok(Json(AnalyzeResponse { extractor: state.extractor.label(), extracted, results }))
}
