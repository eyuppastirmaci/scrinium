use crate::domain::model::{
    ExtractedPage, NewProcessingJob, ProcessingJob, ProcessingJobStatus,
};
use crate::domain::port::{JobStoreError, ProcessingJobRepository};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct SqlxProcessingJobRepository {
    pool: PgPool,
}

impl SqlxProcessingJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ProcessingJobRepository for SqlxProcessingJobRepository {
    async fn find_by_document_id(
        &self,
        document_id: Uuid,
    ) -> Result<Option<ProcessingJob>, JobStoreError> {
        let row = sqlx::query_as::<_, ProcessingJobRow>(
            "SELECT document_id, status, file_name, content_type, size_bytes,
                    storage_object_key, sha256, attempts, last_error,
                    started_at, completed_at, failed_at, created_at, updated_at
             FROM processing_jobs
             WHERE document_id = $1",
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| JobStoreError(format!("find_by_document_id: {e}")))?;

        match row {
            None => Ok(None),
            Some(r) => Ok(Some(r.into_domain()?)),
        }
    }

    async fn start_or_update_received(&self, job: NewProcessingJob) -> Result<(), JobStoreError> {
        sqlx::query(
            "INSERT INTO processing_jobs (document_id, status, file_name, content_type,
                    size_bytes, storage_object_key, sha256, attempts, started_at)
             VALUES ($1, 'RECEIVED', $2, $3, $4, $5, $6, 1, now())
             ON CONFLICT (document_id) DO UPDATE SET
                    status = 'RECEIVED',
                    attempts = processing_jobs.attempts + 1,
                    last_error = NULL,
                    started_at = now(),
                    updated_at = now()",
        )
        .bind(job.document_id)
        .bind(&job.file_name)
        .bind(&job.content_type)
        .bind(job.size_bytes)
        .bind(&job.storage_object_key)
        .bind(&job.sha256)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| JobStoreError(format!("start_or_update_received: {e}")))
    }

    async fn mark_completed(&self, document_id: Uuid) -> Result<(), JobStoreError> {
        sqlx::query(
            "UPDATE processing_jobs
             SET status = 'COMPLETED', completed_at = now(), updated_at = now()
             WHERE document_id = $1",
        )
        .bind(document_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| JobStoreError(format!("mark_completed: {e}")))
    }

    async fn mark_failed(&self, document_id: Uuid, reason: &str) -> Result<(), JobStoreError> {
        sqlx::query(
            "UPDATE processing_jobs
             SET status = 'FAILED', last_error = $2, failed_at = now(), updated_at = now()
             WHERE document_id = $1",
        )
        .bind(document_id)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| JobStoreError(format!("mark_failed: {e}")))
    }

    async fn save_extracted_pages(
        &self,
        document_id: Uuid,
        pages: &[ExtractedPage],
    ) -> Result<(), JobStoreError> {
        if pages.is_empty() {
            return Ok(());
        }

        sqlx::query("DELETE FROM extracted_pages WHERE document_id = $1")
            .bind(document_id)
            .execute(&self.pool)
            .await
            .map_err(|e| JobStoreError(format!("delete old pages: {e}")))?;

        let mut doc_ids: Vec<Uuid> = Vec::with_capacity(pages.len());
        let mut page_numbers: Vec<i32> = Vec::with_capacity(pages.len());
        let mut texts: Vec<&str> = Vec::with_capacity(pages.len());

        for page in pages {
            doc_ids.push(document_id);
            page_numbers.push(page.page_number);
            texts.push(&page.text);
        }

        sqlx::query(
            "INSERT INTO extracted_pages (document_id, page_number, extracted_text)
             SELECT * FROM UNNEST($1::uuid[], $2::int[], $3::text[])",
        )
        .bind(&doc_ids)
        .bind(&page_numbers)
        .bind(&texts)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| JobStoreError(format!("save_extracted_pages: {e}")))
    }
}

#[derive(sqlx::FromRow)]
struct ProcessingJobRow {
    document_id: Uuid,
    status: String,
    file_name: String,
    content_type: String,
    size_bytes: i64,
    storage_object_key: String,
    sha256: String,
    attempts: i32,
    last_error: Option<String>,
    started_at: Option<chrono::DateTime<Utc>>,
    completed_at: Option<chrono::DateTime<Utc>>,
    failed_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl ProcessingJobRow {
    fn into_domain(self) -> Result<ProcessingJob, JobStoreError> {
        let status = ProcessingJobStatus::from_db_str(&self.status)
            .map_err(JobStoreError)?;
        Ok(ProcessingJob {
            document_id: self.document_id,
            status,
            file_name: self.file_name,
            content_type: self.content_type,
            size_bytes: self.size_bytes,
            storage_object_key: self.storage_object_key,
            sha256: self.sha256,
            attempts: self.attempts,
            last_error: self.last_error,
            started_at: self.started_at,
            completed_at: self.completed_at,
            failed_at: self.failed_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
