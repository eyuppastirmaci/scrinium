use crate::domain::model::{DocumentMetadata, NewDocumentMetadata};
use crate::domain::port::{MetadataRepository, MetadataStoreError};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct SqlxMetadataRepository {
    pool: PgPool,
}

impl SqlxMetadataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl MetadataRepository for SqlxMetadataRepository {
    async fn find_by_document_id(
        &self,
        document_id: Uuid,
    ) -> Result<Option<DocumentMetadata>, MetadataStoreError> {
        let row = sqlx::query_as::<_, DocumentMetadataRow>(
            "SELECT document_id, page_count, pdf_created_at, pdf_modified_at,
                    pdf_author, pdf_title, image_captured_at, image_device,
                    image_gps_present, image_gps_redacted, detected_language,
                    metadata_json::text AS metadata_json, created_at, updated_at
             FROM document_metadata
             WHERE document_id = $1",
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| MetadataStoreError(format!("find_metadata_by_document_id: {e}")))?;

        Ok(row.map(DocumentMetadataRow::into_domain))
    }

    async fn upsert(&self, metadata: NewDocumentMetadata) -> Result<(), MetadataStoreError> {
        sqlx::query(
            "INSERT INTO document_metadata (
                    document_id, page_count, pdf_created_at, pdf_modified_at,
                    pdf_author, pdf_title, image_captured_at, image_device,
                    image_gps_present, image_gps_redacted, detected_language,
                    metadata_json
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::jsonb)
             ON CONFLICT (document_id) DO UPDATE SET
                    page_count = EXCLUDED.page_count,
                    pdf_created_at = EXCLUDED.pdf_created_at,
                    pdf_modified_at = EXCLUDED.pdf_modified_at,
                    pdf_author = EXCLUDED.pdf_author,
                    pdf_title = EXCLUDED.pdf_title,
                    image_captured_at = EXCLUDED.image_captured_at,
                    image_device = EXCLUDED.image_device,
                    image_gps_present = EXCLUDED.image_gps_present,
                    image_gps_redacted = EXCLUDED.image_gps_redacted,
                    detected_language = EXCLUDED.detected_language,
                    metadata_json = EXCLUDED.metadata_json,
                    updated_at = now()",
        )
        .bind(metadata.document_id)
        .bind(metadata.page_count)
        .bind(metadata.pdf_created_at)
        .bind(metadata.pdf_modified_at)
        .bind(metadata.pdf_author)
        .bind(metadata.pdf_title)
        .bind(metadata.image_captured_at)
        .bind(metadata.image_device)
        .bind(metadata.image_gps_present)
        .bind(metadata.image_gps_redacted)
        .bind(metadata.detected_language)
        .bind(metadata.metadata_json)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| MetadataStoreError(format!("upsert_document_metadata: {e}")))
    }
}

#[derive(sqlx::FromRow)]
struct DocumentMetadataRow {
    document_id: Uuid,
    page_count: Option<i32>,
    pdf_created_at: Option<DateTime<Utc>>,
    pdf_modified_at: Option<DateTime<Utc>>,
    pdf_author: Option<String>,
    pdf_title: Option<String>,
    image_captured_at: Option<DateTime<Utc>>,
    image_device: Option<String>,
    image_gps_present: bool,
    image_gps_redacted: bool,
    detected_language: Option<String>,
    metadata_json: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl DocumentMetadataRow {
    fn into_domain(self) -> DocumentMetadata {
        DocumentMetadata {
            document_id: self.document_id,
            page_count: self.page_count,
            pdf_created_at: self.pdf_created_at,
            pdf_modified_at: self.pdf_modified_at,
            pdf_author: self.pdf_author,
            pdf_title: self.pdf_title,
            image_captured_at: self.image_captured_at,
            image_device: self.image_device,
            image_gps_present: self.image_gps_present,
            image_gps_redacted: self.image_gps_redacted,
            detected_language: self.detected_language,
            metadata_json: self.metadata_json,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
