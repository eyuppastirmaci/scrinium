use crate::domain::model::ProcessingCompletedEvent;
use crate::domain::port::{EventPublisher, PublishError};
use chrono::Utc;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use std::time::Duration;
use uuid::Uuid;

pub struct KafkaEventPublisher {
    producer: FutureProducer,
}

impl KafkaEventPublisher {
    pub fn new(brokers: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .create()
            .expect("producer creation failed");
        Self { producer }
    }

    async fn publish(
        &self,
        topic: &str,
        document_id: &str,
        event_type: &str,
        payload_json: &str,
    ) -> Result<(), PublishError> {
        let json = serde_json::to_string(&serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "type": event_type,
            "version": 1,
            "timestamp": Utc::now().to_rfc3339(),
            "payload": serde_json::from_str::<serde_json::Value>(payload_json).unwrap()
        }))
        .map_err(|e| PublishError(format!("serialize failed: {e}")))?;

        let record = FutureRecord::to(topic).key(document_id).payload(&json);
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map(|_| ())
            .map_err(|(e, _)| PublishError(e.to_string()))
    }
}

#[async_trait::async_trait]
impl EventPublisher for KafkaEventPublisher {
    async fn processing_completed(&self, event: &ProcessingCompletedEvent) -> Result<(), PublishError> {
        let document_id = event.document_id.to_string();
        let pages: Vec<CompletedPagePayload> = event
            .pages
            .iter()
            .map(|p| CompletedPagePayload {
                page_number: p.page_number,
                text: p.text.clone(),
            })
            .collect();

        let thumbnails: Vec<CompletedThumbnailPayload> = event
            .thumbnails
            .iter()
            .map(|t| CompletedThumbnailPayload {
                size: t.size.suffix().to_uppercase(),
                storage_key: t.storage_key.clone(),
                width: t.width,
                height: t.height,
            })
            .collect();

        let metadata = &event.metadata;
        let metadata_payload = CompletedMetadataPayload {
            page_count: metadata.page_count,
            pdf_created_at: metadata.pdf_created_at.map(|d| d.to_rfc3339()),
            pdf_modified_at: metadata.pdf_modified_at.map(|d| d.to_rfc3339()),
            pdf_author: metadata.pdf_author.clone(),
            pdf_title: metadata.pdf_title.clone(),
            image_captured_at: metadata.image_captured_at.map(|d| d.to_rfc3339()),
            image_device: metadata.image_device.clone(),
            image_gps_present: metadata.image_gps_present,
            image_gps_redacted: metadata.image_gps_redacted,
            detected_language: metadata.detected_language.clone(),
        };

        let payload = serde_json::to_string(&CompletedPayload {
            document_id: document_id.clone(),
            file_name: event.file_name.clone(),
            content_type: event.content_type.clone(),
            created_at: event.created_at.clone(),
            pages,
            metadata: metadata_payload,
            thumbnails,
        })
        .map_err(|e| PublishError(format!("serialize failed: {e}")))?;

        self.publish(
            "document.processing.completed",
            &document_id,
            "document.processing.completed",
            &payload,
        )
        .await
    }

    async fn processing_failed(&self, document_id: &str, reason: &str) -> Result<(), PublishError> {
        let payload = serde_json::to_string(&FailedPayload {
            document_id: document_id.to_string(),
            reason: reason.to_string(),
        })
        .map_err(|e| PublishError(format!("serialize failed: {e}")))?;

        self.publish(
            "document.processing.failed",
            document_id,
            "document.processing.failed",
            &payload,
        )
        .await
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletedPayload {
    document_id: String,
    file_name: String,
    content_type: String,
    created_at: String,
    pages: Vec<CompletedPagePayload>,
    metadata: CompletedMetadataPayload,
    thumbnails: Vec<CompletedThumbnailPayload>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletedPagePayload {
    page_number: i32,
    text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletedMetadataPayload {
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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletedThumbnailPayload {
    size: String,
    storage_key: String,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FailedPayload {
    document_id: String,
    reason: String,
}
