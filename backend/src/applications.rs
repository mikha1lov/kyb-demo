use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    application::{self, ApplicationRow, ApplicationView, Business, Sources, Status},
    domain::DocKind,
    error::ApiError,
    extractor::DocumentFile,
    routes::{AppState, MAX_UPLOAD_BYTES},
    store,
};

const MAX_DOCUMENTS: usize = 10;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/applications", post(create))
        .route("/api/applications/{id}", get(show))
        .route("/api/applications/{id}/business", put(save_business))
        .route("/api/applications/{id}/documents", post(upload))
        .route("/api/applications/{id}/documents/{document}", delete(remove))
        .route("/api/applications/{id}/submit", post(submit))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BusinessPayload {
    business: Business,
    #[serde(default)]
    sources: Sources,
}

fn checked(payload: BusinessPayload) -> Result<(Business, Sources), ApiError> {
    let business = payload.business.normalized();
    business.validate().map_err(|message| ApiError::unprocessable("invalid_business", message))?;
    Ok((business, payload.sources))
}

async fn load(state: &AppState, id: Uuid) -> Result<ApplicationRow, ApiError> {
    store::find(&state.pool, id).await?.ok_or(ApiError::NotFound)
}

async fn editable(state: &AppState, id: Uuid) -> Result<ApplicationRow, ApiError> {
    let application = load(state, id).await?;
    if application.status() == Status::Submitted {
        return Err(ApiError::Conflict("already_submitted"));
    }
    Ok(application)
}

async fn respond(state: &AppState, id: Uuid) -> Result<ApplicationView, ApiError> {
    let application = load(state, id).await?;
    let documents = store::documents(&state.pool, id).await?;
    Ok(application::view(application, documents, Utc::now().date_naive()))
}

async fn create(
    State(state): State<AppState>,
    Json(payload): Json<BusinessPayload>,
) -> Result<(StatusCode, Json<ApplicationView>), ApiError> {
    let (business, sources) = checked(payload)?;
    let application = store::create(&state.pool, &business, &sources).await?;
    Ok((StatusCode::CREATED, Json(respond(&state, application.id).await?)))
}

async fn show(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApplicationView>, ApiError> {
    Ok(Json(respond(&state, id).await?))
}

async fn save_business(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<BusinessPayload>,
) -> Result<Json<ApplicationView>, ApiError> {
    editable(&state, id).await?;
    let (business, sources) = checked(payload)?;
    store::update_business(&state.pool, id, &business, &sources).await?.ok_or(ApiError::Conflict("already_submitted"))?;
    Ok(Json(respond(&state, id).await?))
}

fn kind_label(kind: DocKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|value| value.as_str().map(String::from))
        .unwrap_or_else(|| "other".into())
}

async fn upload(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    mut form: Multipart,
) -> Result<(StatusCode, Json<ApplicationView>), ApiError> {
    let mut file: Option<DocumentFile> = None;
    while let Some(field) = form.next_field().await.map_err(|e| ApiError::BadRequest(e.to_string()))? {
        if field.name() == Some("file") {
            let filename = field.file_name().unwrap_or("document").to_string();
            let mime = field.content_type().unwrap_or("application/octet-stream").to_string();
            let bytes = field.bytes().await.map_err(|e| ApiError::BadRequest(e.to_string()))?.to_vec();
            if bytes.len() > MAX_UPLOAD_BYTES {
                return Err(ApiError::BadRequest("file is larger than 10 MB".into()));
            }
            file = Some(DocumentFile { filename, mime, bytes });
        }
    }
    let file = file.ok_or_else(|| ApiError::BadRequest("file is required".into()))?;

    let application = editable(&state, id).await?;
    if application.business.is_none() {
        return Err(ApiError::unprocessable("business_required", "Business details come first"));
    }
    if store::documents(&state.pool, id).await?.len() >= MAX_DOCUMENTS {
        return Err(ApiError::unprocessable("too_many_documents", "Too many documents"));
    }

    tracing::info!(bytes = file.bytes.len(), mime = %file.mime, "analyzing document");
    let filename = file.filename.clone();
    let size = i32::try_from(file.bytes.len()).unwrap_or(i32::MAX);
    let extracted = state.extractor.extract(file).await?;
    store::insert_document(&state.pool, id, &filename, size, &kind_label(extracted.kind), &extracted).await?;

    Ok((StatusCode::CREATED, Json(respond(&state, id).await?)))
}

async fn remove(
    State(state): State<AppState>,
    Path((id, document)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApplicationView>, ApiError> {
    editable(&state, id).await?;
    if !store::delete_document(&state.pool, id, document).await? {
        return Err(ApiError::NotFound);
    }
    Ok(Json(respond(&state, id).await?))
}

async fn submit(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApplicationView>, ApiError> {
    editable(&state, id).await?;
    if !respond(&state, id).await?.complete {
        return Err(ApiError::unprocessable("incomplete", "Some requirements are still open"));
    }
    store::submit(&state.pool, id).await?.ok_or(ApiError::Conflict("already_submitted"))?;
    Ok(Json(respond(&state, id).await?))
}
