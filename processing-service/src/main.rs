mod adapter;
mod application;
mod config;
mod domain;

use adapter::inbound::kafka::build_consumer;
use adapter::outbound::messaging::KafkaEventPublisher;
use adapter::outbound::persistence::SqlxProcessingJobRepository;
use application::{HandleError, ProcessDocument};
use config::AppConfig;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();

    let db_pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await
        .expect("processing database connection failed");
    println!("processing-service connected to processing database");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("processing database migration failed");
    println!("processing-service database migrations applied");

    let publisher = KafkaEventPublisher::new(&config.kafka_brokers);
    let repository = SqlxProcessingJobRepository::new(db_pool);
    let use_case = ProcessDocument::new(&publisher, &repository);

    let consumer = build_consumer(&config.kafka_brokers, &config.kafka_group_id);
    consumer
        .subscribe(&[&config.kafka_in_topic])
        .expect("subscribe failed");
    println!(
        "processing-service listening on '{}'",
        config.kafka_in_topic
    );

    loop {
        match consumer.recv().await {
            Err(e) => eprintln!("kafka receive error: {e}"),
            Ok(message) => {
                let payload = message.payload().unwrap_or(&[]);
                let commit = match use_case.handle(payload).await {
                    Ok(()) => true,
                    Err(HandleError::Skip(reason)) => {
                        eprintln!("skipping: {reason}");
                        true
                    }
                    Err(HandleError::Retry(reason)) => {
                        eprintln!("will retry: {reason}");
                        false
                    }
                };
                if commit {
                    if let Err(e) = consumer.commit_message(&message, CommitMode::Async) {
                        eprintln!("offset commit failed: {e}");
                    }
                }
            }
        }
    }
}
