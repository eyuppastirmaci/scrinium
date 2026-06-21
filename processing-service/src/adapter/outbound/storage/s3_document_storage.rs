use crate::domain::port::{DocumentStorage, StorageError};
use aws_sdk_s3::Client;

pub struct S3DocumentStorage {
    client: Client,
    bucket: String,
}

impl S3DocumentStorage {
    pub fn new(client: Client, bucket: String) -> Self {
        Self { client, bucket }
    }
}

#[async_trait::async_trait]
impl DocumentStorage for S3DocumentStorage {
    async fn read_document(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError(format!("S3 get_object failed for '{key}': {e}")))?;

        let bytes = output
            .body
            .collect()
            .await
            .map_err(|e| StorageError(format!("S3 read body failed for '{key}': {e}")))?
            .into_bytes()
            .to_vec();

        Ok(bytes)
    }
}
