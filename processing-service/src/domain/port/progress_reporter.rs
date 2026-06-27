use uuid::Uuid;

#[derive(Debug)]
pub struct ProgressError(pub String);

#[async_trait::async_trait]
pub trait ProgressReporter: Send + Sync {
    async fn report(&self, document_id: Uuid, step: &str, progress: i32) -> Result<(), ProgressError>;
}
