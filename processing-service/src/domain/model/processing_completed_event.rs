use crate::domain::model::{ExtractedDocumentMetadata, ExtractedPage, ThumbnailSize};
use uuid::Uuid;

pub struct ProcessingCompletedEvent {
    pub document_id: Uuid,
    pub file_name: String,
    pub pages: Vec<ExtractedPage>,
    pub metadata: ExtractedDocumentMetadata,
    pub thumbnails: Vec<ThumbnailInfo>,
}

pub struct ThumbnailInfo {
    pub size: ThumbnailSize,
    pub storage_key: String,
    pub width: u32,
    pub height: u32,
}
