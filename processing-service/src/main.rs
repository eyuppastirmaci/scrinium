mod adapter;
mod application;
mod contract;
mod domain;

use adapter::{KafkaEventPublisher, build_consumer};
use application::{HandleError, ProcessDocument};
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;

const BROKERS: &str = "localhost:9092";
const IN_TOPIC: &str = "document.uploaded";
const GROUP_ID: &str = "processing-service";

#[tokio::main]
async fn main() {
    // Composition root: build adapters and inject the publisher into the use-case.
    let publisher = KafkaEventPublisher::new(BROKERS);
    let use_case = ProcessDocument::new(&publisher);

    let consumer = build_consumer(BROKERS, GROUP_ID);
    consumer.subscribe(&[IN_TOPIC]).expect("subscribe failed");
    println!("processing-service listening on '{IN_TOPIC}'");

    loop {
        match consumer.recv().await {
            Err(e) => eprintln!("kafka receive error: {e}"),
            Ok(message) => {
                let payload = message.payload().unwrap_or(&[]);
                let commit = match use_case.handle(payload).await {
                    Ok(()) => true,
                    // Bad message: commit so we don't loop on it forever.
                    Err(HandleError::Skip(reason)) => {
                        eprintln!("skipping: {reason}");
                        true
                    }
                    // Transient failure: leave the offset for redelivery.
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
