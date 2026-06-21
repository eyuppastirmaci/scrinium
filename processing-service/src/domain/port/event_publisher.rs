#[derive(Debug)]
pub struct PublishError(pub String);

#[async_trait::async_trait]
pub trait EventPublisher {
    async fn processing_completed(&self, document_id: &str) -> Result<(), PublishError>;
    async fn processing_failed(&self, document_id: &str, reason: &str) -> Result<(), PublishError>;
}
