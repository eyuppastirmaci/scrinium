use std::env;

const DEFAULT_KAFKA_BROKERS: &str = "localhost:9092";
const DEFAULT_KAFKA_IN_TOPIC: &str = "document.uploaded";
const DEFAULT_KAFKA_GROUP_ID: &str = "processing-service";
const DEFAULT_DATABASE_URL: &str = "postgres://scrinium:scrinium@localhost:5433/processing";
const DEFAULT_DB_MAX_CONNECTIONS: u32 = 5;

pub struct AppConfig {
    pub kafka_brokers: String,
    pub kafka_in_topic: String,
    pub kafka_group_id: String,
    pub database_url: String,
    pub db_max_connections: u32,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            kafka_brokers: env::var("PROCESSING_KAFKA_BROKERS")
                .unwrap_or_else(|_| DEFAULT_KAFKA_BROKERS.to_string()),
            kafka_in_topic: env::var("PROCESSING_KAFKA_IN_TOPIC")
                .unwrap_or_else(|_| DEFAULT_KAFKA_IN_TOPIC.to_string()),
            kafka_group_id: env::var("PROCESSING_KAFKA_GROUP_ID")
                .unwrap_or_else(|_| DEFAULT_KAFKA_GROUP_ID.to_string()),
            database_url: env::var("PROCESSING_DATABASE_URL")
                .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string()),
            db_max_connections: env::var("PROCESSING_DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_DB_MAX_CONNECTIONS),
        }
    }
}
