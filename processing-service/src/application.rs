use crate::contract::DocumentUploaded;
use crate::domain::{
    EventPublisher, NewProcessingJob, ProcessingJobRepository, ProcessingJobStatus,
};
use uuid::Uuid;

pub struct ProcessDocument<'a, P: EventPublisher, R: ProcessingJobRepository> {
    publisher: &'a P,
    repository: &'a R,
}

impl<'a, P: EventPublisher, R: ProcessingJobRepository> ProcessDocument<'a, P, R> {
    pub fn new(publisher: &'a P, repository: &'a R) -> Self {
        Self {
            publisher,
            repository,
        }
    }

    /// Handles one document.uploaded payload. Returns Ok(()) if the offset may be
    /// committed; Err if the message should be retried (at-least-once).
    pub async fn handle(&self, raw_payload: &[u8]) -> Result<(), HandleError> {
        let uploaded: DocumentUploaded = serde_json::from_slice(raw_payload)
            .map_err(|e| HandleError::Skip(format!("malformed event: {e}")))?;

        let payload = uploaded.payload;
        let document_id: Uuid = payload
            .document_id
            .parse()
            .map_err(|e| HandleError::Skip(format!("invalid document_id: {e}")))?;

        println!(
            "received document.uploaded for {document_id}: {} ({} bytes, {}, key {}, sha256 {})",
            payload.file_name,
            payload.size_bytes,
            payload.content_type,
            payload.storage_object_key,
            payload.sha256
        );

        let existing = self
            .repository
            .find_by_document_id(document_id)
            .await
            .map_err(|e| HandleError::Retry(e.0))?;

        if let Some(job) = existing {
            if job.status == ProcessingJobStatus::Completed {
                println!("skipping already-completed document {document_id}");
                return Ok(());
            }
        }

        let new_job = NewProcessingJob {
            document_id,
            file_name: payload.file_name,
            content_type: payload.content_type,
            size_bytes: payload.size_bytes,
            storage_object_key: payload.storage_object_key,
            sha256: payload.sha256,
        };

        self.repository
            .start_or_update_received(new_job)
            .await
            .map_err(|e| HandleError::Retry(e.0))?;

        // Phase-1 walking skeleton: "processing" is a no-op.

        self.repository
            .mark_completed(document_id)
            .await
            .map_err(|e| HandleError::Retry(e.0))?;

        self.publisher
            .processing_completed(&document_id.to_string())
            .await
            .map_err(|e| HandleError::Retry(e.0))?;

        println!("published document.processing.completed for {document_id}");
        Ok(())
    }
}

#[derive(Debug)]
pub enum HandleError {
    Skip(String),  // commit the offset (don't reprocess a bad message)
    Retry(String), // leave the offset (reprocess later)
}
