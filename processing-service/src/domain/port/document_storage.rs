#[derive(Debug)]
pub struct StorageError(pub String);

#[async_trait::async_trait]
pub trait DocumentStorage: Send + Sync {
    async fn read_document(&self, key: &str) -> Result<Vec<u8>, StorageError>;
    async fn write_object(&self, key: &str, bytes: &[u8], content_type: &str) -> Result<(), StorageError>;
}
