use crate::contract::{DocumentProcessingCompleted, DocumentProcessingCompletedPayload};
use crate::domain::{EventPublisher, PublishError};
use chrono::Utc;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use uuid::Uuid;

const OUT_TOPIC: &str = "document.processing.completed";
const EVENT_TYPE: &str = "document.processing.completed";
const EVENT_VERSION: u32 = 1;

// Outbound adapter: implements the domain port using Kafka.
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
}

#[async_trait::async_trait]
impl EventPublisher for KafkaEventPublisher {
    async fn processing_completed(&self, document_id: &str) -> Result<(), PublishError> {
        let event = DocumentProcessingCompleted {
            id: Uuid::new_v4().to_string(),
            event_type: EVENT_TYPE.to_string(),
            version: EVENT_VERSION,
            timestamp: Utc::now().to_rfc3339(),
            payload: DocumentProcessingCompletedPayload {
                document_id: document_id.to_string(),
            },
        };

        let json = serde_json::to_string(&event)
            .map_err(|e| PublishError(format!("serialize failed: {e}")))?;

        let record = FutureRecord::to(OUT_TOPIC).key(document_id).payload(&json);
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map(|_| ())
            .map_err(|(e, _)| PublishError(e.to_string()))
    }
}

// Inbound adapter helper: builds the Kafka consumer.
pub fn build_consumer(brokers: &str, group_id: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", "false")
        .create()
        .expect("consumer creation failed")
}
