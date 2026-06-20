use crate::contract::{DocumentProcessingCompleted, DocumentProcessingCompletedPayload};
use crate::domain::{
    EventPublisher, JobStoreError, NewProcessingJob, ProcessingJob, ProcessingJobRepository,
    ProcessingJobStatus, PublishError,
};
use chrono::Utc;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::{FutureProducer, FutureRecord};
use sqlx::PgPool;
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

// Outbound adapter: implements the job repository port using PostgreSQL via sqlx.
pub struct SqlxProcessingJobRepository {
    pool: PgPool,
}

impl SqlxProcessingJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ProcessingJobRepository for SqlxProcessingJobRepository {
    async fn find_by_document_id(
        &self,
        document_id: Uuid,
    ) -> Result<Option<ProcessingJob>, JobStoreError> {
        let row = sqlx::query_as::<_, ProcessingJobRow>(
            "SELECT document_id, status, file_name, content_type, size_bytes,
                    storage_object_key, sha256, attempts, last_error,
                    started_at, completed_at, failed_at, created_at, updated_at
             FROM processing_jobs
             WHERE document_id = $1",
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| JobStoreError(format!("find_by_document_id: {e}")))?;

        match row {
            None => Ok(None),
            Some(r) => Ok(Some(r.into_domain()?)),
        }
    }

    async fn start_or_update_received(&self, job: NewProcessingJob) -> Result<(), JobStoreError> {
        sqlx::query(
            "INSERT INTO processing_jobs (document_id, status, file_name, content_type,
                    size_bytes, storage_object_key, sha256, attempts, started_at)
             VALUES ($1, 'RECEIVED', $2, $3, $4, $5, $6, 1, now())
             ON CONFLICT (document_id) DO UPDATE SET
                    status = 'RECEIVED',
                    attempts = processing_jobs.attempts + 1,
                    last_error = NULL,
                    started_at = now(),
                    updated_at = now()",
        )
        .bind(job.document_id)
        .bind(&job.file_name)
        .bind(&job.content_type)
        .bind(job.size_bytes)
        .bind(&job.storage_object_key)
        .bind(&job.sha256)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| JobStoreError(format!("start_or_update_received: {e}")))
    }

    async fn mark_completed(&self, document_id: Uuid) -> Result<(), JobStoreError> {
        sqlx::query(
            "UPDATE processing_jobs
             SET status = 'COMPLETED', completed_at = now(), updated_at = now()
             WHERE document_id = $1",
        )
        .bind(document_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| JobStoreError(format!("mark_completed: {e}")))
    }

    async fn mark_failed(&self, document_id: Uuid, reason: &str) -> Result<(), JobStoreError> {
        sqlx::query(
            "UPDATE processing_jobs
             SET status = 'FAILED', last_error = $2, failed_at = now(), updated_at = now()
             WHERE document_id = $1",
        )
        .bind(document_id)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| JobStoreError(format!("mark_failed: {e}")))
    }
}

#[derive(sqlx::FromRow)]
struct ProcessingJobRow {
    document_id: Uuid,
    status: String,
    file_name: String,
    content_type: String,
    size_bytes: i64,
    storage_object_key: String,
    sha256: String,
    attempts: i32,
    last_error: Option<String>,
    started_at: Option<chrono::DateTime<Utc>>,
    completed_at: Option<chrono::DateTime<Utc>>,
    failed_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl ProcessingJobRow {
    fn into_domain(self) -> Result<ProcessingJob, JobStoreError> {
        let status = ProcessingJobStatus::from_db_str(&self.status)?;
        Ok(ProcessingJob {
            document_id: self.document_id,
            status,
            file_name: self.file_name,
            content_type: self.content_type,
            size_bytes: self.size_bytes,
            storage_object_key: self.storage_object_key,
            sha256: self.sha256,
            attempts: self.attempts,
            last_error: self.last_error,
            started_at: self.started_at,
            completed_at: self.completed_at,
            failed_at: self.failed_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
