#[derive(Debug)]
pub struct StorageError(pub String);

#[async_trait::async_trait]
pub trait DocumentStorage {
    async fn read_document(&self, key: &str) -> Result<Vec<u8>, StorageError>;
}
