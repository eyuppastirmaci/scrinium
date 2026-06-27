use crate::domain::model::{DocumentMetadata, ExtractedPage};
use crate::domain::port::{MetadataRepository, ProcessingJobRepository};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router, routing::get};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct HttpState {
    metadata_repository: Arc<dyn MetadataRepository>,
    processing_job_repository: Arc<dyn ProcessingJobRepository>,
}

impl HttpState {
    pub fn new(
        metadata_repository: Arc<dyn MetadataRepository>,
        processing_job_repository: Arc<dyn ProcessingJobRepository>,
    ) -> Self {
        Self {
            metadata_repository,
            processing_job_repository,
        }
    }
}

pub fn router(
    metadata_repository: Arc<dyn MetadataRepository>,
    processing_job_repository: Arc<dyn ProcessingJobRepository>,
) -> Router {
    let state = HttpState::new(metadata_repository, processing_job_repository);

    Router::new()
        .route(
            "/api/documents/{document_id}/metadata",
            get(get_document_metadata),
        )
        .route(
            "/documents/{document_id}/metadata",
            get(get_document_metadata),
        )
        .route("/api/documents/{document_id}/text", get(get_document_text))
        .route("/documents/{document_id}/text", get(get_document_text))
        .with_state(state)
}

async fn get_document_metadata(
    State(state): State<HttpState>,
    Path(document_id): Path<String>,
) -> Response {
    let document_id = match Uuid::parse_str(&document_id) {
        Ok(document_id) => document_id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "invalid_document_id",
                    message: "document_id must be a UUID",
                }),
            )
                .into_response();
        }
    };

    match state
        .metadata_repository
        .find_by_document_id(document_id)
        .await
    {
        Ok(Some(metadata)) => Json(DocumentMetadataResponse::from(metadata)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "metadata_not_found",
                message: "metadata was not found for this document",
            }),
        )
            .into_response(),
        Err(e) => {
            eprintln!(
                "metadata retrieval failed for document {document_id}: {}",
                e.0
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "metadata_retrieval_failed",
                    message: "metadata could not be retrieved",
                }),
            )
                .into_response()
        }
    }
}

async fn get_document_text(
    State(state): State<HttpState>,
    Path(document_id): Path<String>,
) -> Response {
    let document_id = match Uuid::parse_str(&document_id) {
        Ok(document_id) => document_id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "invalid_document_id",
                    message: "document_id must be a UUID",
                }),
            )
                .into_response();
        }
    };

    match state
        .processing_job_repository
        .find_extracted_pages(document_id)
        .await
    {
        Ok(pages) if pages.is_empty() => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "extracted_text_not_found",
                message: "extracted text was not found for this document",
            }),
        )
            .into_response(),
        Ok(pages) => Json(DocumentTextResponse::new(document_id, pages)).into_response(),
        Err(e) => {
            eprintln!(
                "extracted text retrieval failed for document {document_id}: {}",
                e.0
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "extracted_text_retrieval_failed",
                    message: "extracted text could not be retrieved",
                }),
            )
                .into_response()
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentMetadataResponse {
    document_id: String,
    page_count: Option<i32>,
    pdf_created_at: Option<String>,
    pdf_modified_at: Option<String>,
    pdf_author: Option<String>,
    pdf_title: Option<String>,
    image_captured_at: Option<String>,
    image_device: Option<String>,
    image_gps_present: bool,
    image_gps_redacted: bool,
    detected_language: Option<String>,
    metadata: Value,
    created_at: String,
    updated_at: String,
}

impl From<DocumentMetadata> for DocumentMetadataResponse {
    fn from(metadata: DocumentMetadata) -> Self {
        Self {
            document_id: metadata.document_id.to_string(),
            page_count: metadata.page_count,
            pdf_created_at: metadata.pdf_created_at.map(|value| value.to_rfc3339()),
            pdf_modified_at: metadata.pdf_modified_at.map(|value| value.to_rfc3339()),
            pdf_author: metadata.pdf_author,
            pdf_title: metadata.pdf_title,
            image_captured_at: metadata.image_captured_at.map(|value| value.to_rfc3339()),
            image_device: metadata.image_device,
            image_gps_present: metadata.image_gps_present,
            image_gps_redacted: metadata.image_gps_redacted,
            detected_language: metadata.detected_language,
            metadata: serde_json::from_str(&metadata.metadata_json).unwrap_or(Value::Null),
            created_at: metadata.created_at.to_rfc3339(),
            updated_at: metadata.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentTextResponse {
    document_id: String,
    pages: Vec<ExtractedPageResponse>,
    combined_text: String,
}

impl DocumentTextResponse {
    fn new(document_id: Uuid, pages: Vec<ExtractedPage>) -> Self {
        let combined_text = pages
            .iter()
            .map(|page| page.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        Self {
            document_id: document_id.to_string(),
            pages: pages.into_iter().map(ExtractedPageResponse::from).collect(),
            combined_text,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExtractedPageResponse {
    page_number: i32,
    text: String,
}

impl From<ExtractedPage> for ExtractedPageResponse {
    fn from(page: ExtractedPage) -> Self {
        Self {
            page_number: page.page_number,
            text: page.text,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn document_metadata_response_parses_metadata_json() {
        let response = DocumentMetadataResponse::from(DocumentMetadata {
            document_id: Uuid::nil(),
            page_count: Some(2),
            pdf_created_at: Some(Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap()),
            pdf_modified_at: None,
            pdf_author: Some("Ayse".to_string()),
            pdf_title: Some("Invoice".to_string()),
            image_captured_at: None,
            image_device: None,
            image_gps_present: false,
            image_gps_redacted: false,
            detected_language: Some("tur".to_string()),
            metadata_json: r#"{"pdf":{"hasDocumentInfo":true}}"#.to_string(),
            created_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 6).unwrap(),
        });

        assert_eq!(response.page_count, Some(2));
        assert_eq!(response.metadata["pdf"]["hasDocumentInfo"], true);
        assert_eq!(response.detected_language.as_deref(), Some("tur"));
    }

    #[test]
    fn document_text_response_combines_pages_for_copy_and_download() {
        let response = DocumentTextResponse::new(
            Uuid::nil(),
            vec![
                ExtractedPage {
                    page_number: 1,
                    text: "First page text".to_string(),
                },
                ExtractedPage {
                    page_number: 2,
                    text: "Second page text".to_string(),
                },
            ],
        );

        assert_eq!(response.document_id, Uuid::nil().to_string());
        assert_eq!(response.pages.len(), 2);
        assert_eq!(response.pages[0].page_number, 1);
        assert_eq!(
            response.combined_text,
            "First page text\n\nSecond page text"
        );
    }
}
