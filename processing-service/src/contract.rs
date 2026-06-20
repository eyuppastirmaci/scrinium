use serde::{Deserialize, Serialize};

// --- Inbound: deserialize only what we use; unknown fields ignored (forward-compat). ---
#[derive(Debug, Deserialize)]
pub struct DocumentUploaded {
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

// --- Outbound: serialized to exactly the JSON Schema shape. ---
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentProcessingCompleted {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub version: u32,
    pub timestamp: String,
    pub payload: DocumentProcessingCompletedPayload,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentProcessingCompletedPayload {
    pub document_id: String,
}
