use crate::domain::model::ThumbnailSize;
use uuid::Uuid;

#[derive(Debug)]
pub struct ThumbnailStoreError(pub String);

#[derive(Debug, Clone)]
pub struct ThumbnailRecord {
    pub document_id: Uuid,
    pub size: ThumbnailSize,
    pub storage_key: String,
    pub width: i32,
    pub height: i32,
}

#[async_trait::async_trait]
pub trait ThumbnailRepository: Send + Sync {
    async fn upsert(
        &self,
        document_id: Uuid,
        size: ThumbnailSize,
        storage_key: &str,
        width: i32,
        height: i32,
    ) -> Result<(), ThumbnailStoreError>;

    async fn find_by_document_id_and_size(
        &self,
        document_id: Uuid,
        size: ThumbnailSize,
    ) -> Result<Option<ThumbnailRecord>, ThumbnailStoreError>;
}
