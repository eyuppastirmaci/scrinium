use std::env;

const DEFAULT_KAFKA_BROKERS: &str = "localhost:9092";
const DEFAULT_KAFKA_IN_TOPIC: &str = "document.uploaded";
const DEFAULT_KAFKA_GROUP_ID: &str = "processing-service";
const DEFAULT_DATABASE_URL: &str = "postgres://scrinium:scrinium@localhost:5433/processing";
const DEFAULT_DB_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_STORAGE_ENDPOINT: &str = "http://localhost:9000";
const DEFAULT_STORAGE_ACCESS_KEY: &str = "minioadmin";
const DEFAULT_STORAGE_SECRET_KEY: &str = "minioadmin";
const DEFAULT_STORAGE_BUCKET: &str = "documents";
const DEFAULT_TESSERACT_PATH: &str = "tesseract";
const DEFAULT_TESSERACT_LANGUAGES: &str = "tur+eng";
const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:8091";

pub struct AppConfig {
    pub kafka_brokers: String,
    pub kafka_in_topic: String,
    pub kafka_group_id: String,
    pub database_url: String,
    pub db_max_connections: u32,
    pub storage_endpoint: String,
    pub storage_access_key: String,
    pub storage_secret_key: String,
    pub storage_bucket: String,
    pub tesseract_path: String,
    pub tesseract_languages: String,
    pub http_addr: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        match dotenvy::dotenv() {
            Ok(path) => println!("loaded .env from {}", path.display()),
            Err(e) => println!(".env not loaded: {e}"),
        }

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
            storage_endpoint: env::var("PROCESSING_STORAGE_ENDPOINT")
                .unwrap_or_else(|_| DEFAULT_STORAGE_ENDPOINT.to_string()),
            storage_access_key: env::var("PROCESSING_STORAGE_ACCESS_KEY")
                .unwrap_or_else(|_| DEFAULT_STORAGE_ACCESS_KEY.to_string()),
            storage_secret_key: env::var("PROCESSING_STORAGE_SECRET_KEY")
                .unwrap_or_else(|_| DEFAULT_STORAGE_SECRET_KEY.to_string()),
            storage_bucket: env::var("PROCESSING_STORAGE_BUCKET")
                .unwrap_or_else(|_| DEFAULT_STORAGE_BUCKET.to_string()),
            tesseract_path: env::var("PROCESSING_TESSERACT_PATH")
                .unwrap_or_else(|_| DEFAULT_TESSERACT_PATH.to_string()),
            tesseract_languages: env::var("PROCESSING_TESSERACT_LANGUAGES")
                .unwrap_or_else(|_| DEFAULT_TESSERACT_LANGUAGES.to_string()),
            http_addr: env::var("PROCESSING_HTTP_ADDR")
                .unwrap_or_else(|_| DEFAULT_HTTP_ADDR.to_string()),
        }
    }
}
