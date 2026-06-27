use crate::domain::model::ProcessingCompletedEvent;

#[derive(Debug)]
pub struct PublishError(pub String);

#[async_trait::async_trait]
pub trait EventPublisher {
    async fn processing_completed(&self, event: &ProcessingCompletedEvent) -> Result<(), PublishError>;
    async fn processing_failed(&self, document_id: &str, reason: &str) -> Result<(), PublishError>;
}
