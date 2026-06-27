use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedDocumentMetadata {
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
    pub metadata_json: Value,
}

impl Default for ExtractedDocumentMetadata {
    fn default() -> Self {
        Self {
            page_count: None,
            pdf_created_at: None,
            pdf_modified_at: None,
            pdf_author: None,
            pdf_title: None,
            image_captured_at: None,
            image_device: None,
            image_gps_present: false,
            image_gps_redacted: false,
            detected_language: None,
            metadata_json: Value::Object(Default::default()),
        }
    }
}

impl ExtractedDocumentMetadata {
    pub fn is_empty(&self) -> bool {
        self.page_count.is_none()
            && self.pdf_created_at.is_none()
            && self.pdf_modified_at.is_none()
            && self.pdf_author.is_none()
            && self.pdf_title.is_none()
            && self.image_captured_at.is_none()
            && self.image_device.is_none()
            && !self.image_gps_present
            && !self.image_gps_redacted
            && self.detected_language.is_none()
            && self
                .metadata_json
                .as_object()
                .is_some_and(|object| object.is_empty())
    }
}
