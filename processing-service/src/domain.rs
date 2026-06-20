// Driven port: the application announces a completed processing through this,
// without knowing the transport (Kafka, etc.).
#[async_trait::async_trait]
pub trait EventPublisher {
    async fn processing_completed(&self, document_id: &str) -> Result<(), PublishError>;
}

#[derive(Debug)]
pub struct PublishError(pub String);
