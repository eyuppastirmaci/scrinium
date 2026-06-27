use crate::domain::model::{ExtractedPage, NewProcessingJob, ProcessingJob};
use uuid::Uuid;

#[derive(Debug)]
pub struct JobStoreError(pub String);

#[async_trait::async_trait]
pub trait ProcessingJobRepository: Send + Sync {
    async fn find_by_document_id(
        &self,
        document_id: Uuid,
    ) -> Result<Option<ProcessingJob>, JobStoreError>;

    async fn start_or_update_received(&self, job: NewProcessingJob) -> Result<(), JobStoreError>;

    async fn mark_completed(&self, document_id: Uuid) -> Result<(), JobStoreError>;

    async fn mark_failed(&self, document_id: Uuid, reason: &str) -> Result<(), JobStoreError>;

    async fn save_extracted_pages(
        &self,
        document_id: Uuid,
        pages: &[ExtractedPage],
    ) -> Result<(), JobStoreError>;

    async fn find_extracted_pages(
        &self,
        document_id: Uuid,
    ) -> Result<Vec<ExtractedPage>, JobStoreError>;
}
