use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use chrono::{Duration, Utc};
use kyb_backend::{
    domain::{DocKind, Extracted, Field},
    extractor::{DocumentExtractor, DocumentFile, ExtractError},
    routes::{AppState, router},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

struct Stub;

fn field(value: Option<&str>) -> Field {
    Field { value: value.map(String::from), confidence: 0.95, page: Some(1) }
}

fn extracted(kind: DocKind, period_end: Option<String>, issued: Option<&str>) -> Extracted {
    Extracted {
        kind,
        legal_name: field(Some("TOKOEV ALIYAR BEKOVICH PE")),
        registration_number: field(Some("20000000000001")),
        address: field(Some("KYRGYZ REPUBLIC, BISHKEK, UL. TESTOVAYA 10, APT 5")),
        period_start: field(None),
        period_end: Field { value: period_end, confidence: 0.9, page: Some(1) },
        issue_date: field(issued),
        notes: None,
    }
}

#[async_trait]
impl DocumentExtractor for Stub {
    async fn extract(&self, file: DocumentFile) -> Result<Extracted, ExtractError> {
        let recent = (Utc::now() - Duration::days(10)).date_naive().to_string();
        match file.filename.as_str() {
            "cert.pdf" => Ok(extracted(DocKind::Registration, None, Some("2023-03-14"))),
            "statement.pdf" => Ok(extracted(DocKind::BankStatement, Some(recent), None)),
            "old.pdf" => Ok(extracted(DocKind::BankStatement, Some("2025-08-31".into()), None)),
            other => Err(ExtractError::Unsupported(other.into())),
        }
    }

    fn label(&self) -> String {
        "stub".into()
    }
}

fn app(pool: PgPool) -> Router {
    router(AppState { extractor: Arc::new(Stub), pool })
}

fn business(name: &str) -> Value {
    json!({
        "business": {
            "country": "kg",
            "entityType": "sole_proprietor",
            "legalName": name,
            "registrationNumber": "20000000000001",
            "address": "Bishkek, Testovaya street 10, apt. 5"
        }
    })
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn json_call(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri).header(header::CONTENT_TYPE, "application/json");
    let body = body.map(|b| Body::from(b.to_string())).unwrap_or_else(Body::empty);
    send(app, builder.body(body).unwrap()).await
}

async fn upload(app: &Router, id: &str, filename: &str) -> (StatusCode, Value) {
    let boundary = "kybboundary";
    let mut body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: application/pdf\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(b"%PDF-stub");
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/applications/{id}/documents"))
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(body))
        .unwrap();
    send(app, request).await
}

async fn create(app: &Router, name: &str) -> String {
    let (status, body) = json_call(app, "POST", "/api/applications", Some(business(name))).await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_string()
}

fn covered(body: &Value) -> Vec<String> {
    body["checklist"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| !item["documentId"].is_null())
        .map(|item| item["requirement"].as_str().unwrap().to_string())
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn creates_and_reads_an_application(pool: PgPool) {
    let app = app(pool);
    let id = create(&app, "Tokoev Aliyar Bekovich").await;
    let (status, body) = json_call(&app, "GET", &format!("/api/applications/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "draft");
    assert_eq!(body["business"]["country"], "KG");
    assert_eq!(body["complete"], false);
    assert_eq!(body["documents"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn rejects_an_invalid_business(pool: PgPool) {
    let app = app(pool);
    let mut payload = business("Tokoev Aliyar Bekovich");
    payload["business"]["country"] = json!("KGZ");
    let (status, body) = json_call(&app, "POST", "/api/applications", Some(payload)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "invalid_business");
}

#[sqlx::test(migrations = "./migrations")]
async fn unknown_application_is_not_found(pool: PgPool) {
    let app = app(pool);
    let (status, body) = json_call(&app, "GET", "/api/applications/2f3b6d6e-0000-4000-8000-000000000000", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_name_difference_is_fixed_by_taking_the_document_value(pool: PgPool) {
    let app = app(pool);
    let id = create(&app, "Aliyar Tokoev PE").await;

    let (status, body) = upload(&app, &id, "cert.pdf").await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(covered(&body).is_empty());
    let issues = &body["documents"][0]["results"][0]["comparison"]["issues"];
    assert_eq!(issues[0]["code"], "name_partial");

    let mut fixed = business("TOKOEV ALIYAR BEKOVICH PE");
    fixed["sources"] = json!({"legalName": "document"});
    let (status, body) = json_call(&app, "PUT", &format!("/api/applications/{id}/business"), Some(fixed)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(covered(&body), vec!["formation"]);
    assert_eq!(body["sources"]["legalName"], "document");
}

#[sqlx::test(migrations = "./migrations")]
async fn one_statement_covers_source_of_funds_and_address(pool: PgPool) {
    let app = app(pool);
    let id = create(&app, "Tokoev Aliyar Bekovich").await;
    let (_, body) = upload(&app, &id, "statement.pdf").await;
    assert_eq!(covered(&body), vec!["source_of_funds", "proof_of_address"]);
    let (_, body) = upload(&app, &id, "old.pdf").await;
    assert_eq!(covered(&body), vec!["source_of_funds", "proof_of_address"]);
    assert_eq!(body["documents"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn submitting_needs_every_requirement_and_locks_the_application(pool: PgPool) {
    let app = app(pool);
    let id = create(&app, "Tokoev Aliyar Bekovich").await;
    let submit = format!("/api/applications/{id}/submit");

    let (status, body) = json_call(&app, "POST", &submit, None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "incomplete");

    upload(&app, &id, "cert.pdf").await;
    let (_, body) = upload(&app, &id, "statement.pdf").await;
    assert_eq!(body["complete"], true);

    let (status, body) = json_call(&app, "POST", &submit, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "submitted");
    assert!(!body["submittedAt"].is_null());

    let document = body["documents"][0]["id"].as_str().unwrap().to_string();
    let (status, _) = json_call(&app, "PUT", &format!("/api/applications/{id}/business"), Some(business("Another Name"))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = upload(&app, &id, "cert.pdf").await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = json_call(&app, "DELETE", &format!("/api/applications/{id}/documents/{document}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, body) = json_call(&app, "POST", &submit, None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "already_submitted");
}

#[sqlx::test(migrations = "./migrations")]
async fn deletes_a_document(pool: PgPool) {
    let app = app(pool);
    let id = create(&app, "Tokoev Aliyar Bekovich").await;
    let (_, body) = upload(&app, &id, "cert.pdf").await;
    let document = body["documents"][0]["id"].as_str().unwrap().to_string();
    let uri = format!("/api/applications/{id}/documents/{document}");

    let (status, body) = json_call(&app, "DELETE", &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["documents"].as_array().unwrap().len(), 0);

    let (status, _) = json_call(&app, "DELETE", &uri, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn unsupported_files_are_rejected(pool: PgPool) {
    let app = app(pool);
    let id = create(&app, "Tokoev Aliyar Bekovich").await;
    let (status, body) = upload(&app, &id, "notes.txt").await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(body["error"], "unsupported_file");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_number_of_documents_is_limited(pool: PgPool) {
    let app = app(pool);
    let id = create(&app, "Tokoev Aliyar Bekovich").await;
    for _ in 0..10 {
        let (status, _) = upload(&app, &id, "cert.pdf").await;
        assert_eq!(status, StatusCode::CREATED);
    }
    let (status, body) = upload(&app, &id, "cert.pdf").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "too_many_documents");
}
