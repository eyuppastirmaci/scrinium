use crate::domain::port::{ProgressError, ProgressReporter};
use redis::AsyncCommands;
use uuid::Uuid;

pub struct RedisProgressReporter {
    client: redis::Client,
}

impl RedisProgressReporter {
    pub fn new(redis_url: &str) -> Result<Self, ProgressError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| ProgressError(format!("redis connection failed: {e}")))?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl ProgressReporter for RedisProgressReporter {
    async fn report(&self, document_id: Uuid, step: &str, progress: i32) -> Result<(), ProgressError> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| ProgressError(format!("redis connect: {e}")))?;

        let key = format!("doc:progress:{document_id}");
        let channel = "doc:progress";

        let message = serde_json::json!({
            "documentId": document_id.to_string(),
            "step": step,
            "progress": progress,
        })
        .to_string();

        // Snapshot for late joiners.
        let _: () = conn.hset(&key, "step", step).await
            .map_err(|e| ProgressError(format!("redis hset: {e}")))?;
        let _: () = conn.hset(&key, "progress", progress).await
            .map_err(|e| ProgressError(format!("redis hset: {e}")))?;
        let _: () = conn.expire(&key, 300).await
            .map_err(|e| ProgressError(format!("redis expire: {e}")))?;

        // Real-time push.
        let _: () = conn.publish(channel, &message).await
            .map_err(|e| ProgressError(format!("redis publish: {e}")))?;

        Ok(())
    }
}
