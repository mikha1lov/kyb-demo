use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::{
    application::{ApplicationRow, Business, DocumentRow, Sources},
    domain::Extracted,
};

pub async fn create(pool: &PgPool, business: &Business, sources: &Sources) -> sqlx::Result<ApplicationRow> {
    sqlx::query_as(
        "insert into applications (business, sources) values ($1, $2)
         returning id, status, business, sources, submitted_at, created_at, updated_at",
    )
    .bind(Json(business))
    .bind(Json(sources))
    .fetch_one(pool)
    .await
}

pub async fn find(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<ApplicationRow>> {
    sqlx::query_as(
        "select id, status, business, sources, submitted_at, created_at, updated_at
         from applications where id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn documents(pool: &PgPool, application_id: Uuid) -> sqlx::Result<Vec<DocumentRow>> {
    sqlx::query_as(
        "select id, application_id, filename, size_bytes, kind, extracted, created_at
         from documents where application_id = $1 order by created_at, id",
    )
    .bind(application_id)
    .fetch_all(pool)
    .await
}

pub async fn update_business(
    pool: &PgPool,
    id: Uuid,
    business: &Business,
    sources: &Sources,
) -> sqlx::Result<Option<ApplicationRow>> {
    sqlx::query_as(
        "update applications set business = $2, sources = $3, updated_at = now()
         where id = $1 and status = 'draft'
         returning id, status, business, sources, submitted_at, created_at, updated_at",
    )
    .bind(id)
    .bind(Json(business))
    .bind(Json(sources))
    .fetch_optional(pool)
    .await
}

pub async fn insert_document(
    pool: &PgPool,
    application_id: Uuid,
    filename: &str,
    size_bytes: i32,
    kind: &str,
    extracted: &Extracted,
) -> sqlx::Result<DocumentRow> {
    sqlx::query_as(
        "insert into documents (application_id, filename, size_bytes, kind, extracted)
         values ($1, $2, $3, $4, $5)
         returning id, application_id, filename, size_bytes, kind, extracted, created_at",
    )
    .bind(application_id)
    .bind(filename)
    .bind(size_bytes)
    .bind(kind)
    .bind(Json(extracted))
    .fetch_one(pool)
    .await
}

pub async fn delete_document(pool: &PgPool, application_id: Uuid, document_id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query("delete from documents where id = $1 and application_id = $2")
        .bind(document_id)
        .bind(application_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn submit(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<ApplicationRow>> {
    sqlx::query_as(
        "update applications set status = 'submitted', submitted_at = now(), updated_at = now()
         where id = $1 and status = 'draft'
         returning id, status, business, sources, submitted_at, created_at, updated_at",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
