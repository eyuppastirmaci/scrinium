use std::path::Path;

#[derive(Debug)]
pub struct OcrError(pub String);

#[async_trait::async_trait]
pub trait OcrEngine: Send + Sync {
    async fn recognize(&self, image_path: &Path) -> Result<String, OcrError>;
}
