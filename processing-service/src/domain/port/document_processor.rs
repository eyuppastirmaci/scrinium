use crate::domain::model::ProcessingResult;

#[derive(Debug)]
pub struct ProcessingError(pub String);

#[async_trait::async_trait]
pub trait DocumentProcessor: Send + Sync {
    async fn process(&self, content: &[u8]) -> Result<ProcessingResult, ProcessingError>;
}
