use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DocumentUploaded {
    pub timestamp: String,
    pub payload: DocumentUploadedPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentUploadedPayload {
    pub document_id: String,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_object_key: String,
    pub sha256: String,
}
