use crate::domain::model::{DocumentMetadata, NewDocumentMetadata};
use uuid::Uuid;

#[derive(Debug)]
pub struct MetadataStoreError(pub String);

#[async_trait::async_trait]
pub trait MetadataRepository: Send + Sync {
    async fn find_by_document_id(
        &self,
        document_id: Uuid,
    ) -> Result<Option<DocumentMetadata>, MetadataStoreError>;

    async fn upsert(&self, metadata: NewDocumentMetadata) -> Result<(), MetadataStoreError>;
}
