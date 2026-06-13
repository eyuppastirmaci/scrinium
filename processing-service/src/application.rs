use crate::contract::DocumentUploaded;
use crate::domain::EventPublisher;

pub struct ProcessDocument<'a, P: EventPublisher> {
    publisher: &'a P,
}

impl<'a, P: EventPublisher> ProcessDocument<'a, P> {
    pub fn new(publisher: &'a P) -> Self {
        Self { publisher }
    }

    /// Handles one document.uploaded payload. Returns Ok(()) if the offset may be
    /// committed; Err if the message should be retried (at-least-once).
    pub async fn handle(&self, raw_payload: &[u8]) -> Result<(), HandleError> {
        // Malformed/unknown message: skip (commit) to avoid a poison-pill loop.
        let uploaded: DocumentUploaded = serde_json::from_slice(raw_payload)
            .map_err(|e| HandleError::Skip(format!("malformed event: {e}")))?;

        let document_id = uploaded.payload.document_id;
        println!("received document.uploaded for {document_id}");

        // Phase-1 walking skeleton: "processing" is a no-op.

        self.publisher
            .processing_completed(&document_id)
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