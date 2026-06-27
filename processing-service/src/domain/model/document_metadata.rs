use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentMetadata {
    pub document_id: Uuid,
    pub page_count: Option<i32>,
    pub pdf_created_at: Option<DateTime<Utc>>,
    pub pdf_modified_at: Option<DateTime<Utc>>,
    pub pdf_author: Option<String>,
    pub pdf_title: Option<String>,
    pub image_captured_at: Option<DateTime<Utc>>,
    pub image_device: Option<String>,
    pub image_gps_present: bool,
    pub image_gps_redacted: bool,
    pub detected_language: Option<String>,
    pub metadata_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewDocumentMetadata {
    pub document_id: Uuid,
    pub page_count: Option<i32>,
    pub pdf_created_at: Option<DateTime<Utc>>,
    pub pdf_modified_at: Option<DateTime<Utc>>,
    pub pdf_author: Option<String>,
    pub pdf_title: Option<String>,
    pub image_captured_at: Option<DateTime<Utc>>,
    pub image_device: Option<String>,
    pub image_gps_present: bool,
    pub image_gps_redacted: bool,
    pub detected_language: Option<String>,
    pub metadata_json: String,
}
