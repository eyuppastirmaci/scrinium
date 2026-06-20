use chrono::{DateTime, Utc};
use uuid::Uuid;

// Driven port: the application announces a completed processing through this,
// without knowing the transport (Kafka, etc.).
#[async_trait::async_trait]
pub trait EventPublisher {
    async fn processing_completed(&self, document_id: &str) -> Result<(), PublishError>;
}

#[derive(Debug)]
pub struct PublishError(pub String);

// Driven port: the application records durable processing state through this,
// without knowing the database technology.
#[async_trait::async_trait]
pub trait ProcessingJobRepository {
    async fn find_by_document_id(
        &self,
        document_id: Uuid,
    ) -> Result<Option<ProcessingJob>, JobStoreError>;

    async fn start_or_update_received(&self, job: NewProcessingJob) -> Result<(), JobStoreError>;

    async fn mark_completed(&self, document_id: Uuid) -> Result<(), JobStoreError>;

    async fn mark_failed(&self, document_id: Uuid, reason: &str) -> Result<(), JobStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessingJob {
    pub document_id: Uuid,
    pub status: ProcessingJobStatus,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_object_key: String,
    pub sha256: String,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProcessingJob {
    pub document_id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_object_key: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingJobStatus {
    Received,
    Processing,
    Completed,
    Failed,
}

impl ProcessingJobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "RECEIVED",
            Self::Processing => "PROCESSING",
            Self::Completed => "COMPLETED",
            Self::Failed => "FAILED",
        }
    }

    pub fn from_db_str(s: &str) -> Result<Self, JobStoreError> {
        match s {
            "RECEIVED" => Ok(Self::Received),
            "PROCESSING" => Ok(Self::Processing),
            "COMPLETED" => Ok(Self::Completed),
            "FAILED" => Ok(Self::Failed),
            other => Err(JobStoreError(format!("unknown job status: {other}"))),
        }
    }
}

#[derive(Debug)]
pub struct JobStoreError(pub String);
