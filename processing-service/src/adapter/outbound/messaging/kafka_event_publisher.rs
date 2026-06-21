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

    async fn publish(&self, topic: &str, document_id: &str, event_type: &str, payload_json: &str) -> Result<(), PublishError> {
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
    async fn processing_completed(&self, document_id: &str) -> Result<(), PublishError> {
        let payload = serde_json::to_string(&CompletedPayload {
            document_id: document_id.to_string(),
        })
        .map_err(|e| PublishError(format!("serialize failed: {e}")))?;

        self.publish(
            "document.processing.completed",
            document_id,
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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FailedPayload {
    document_id: String,
    reason: String,
}
