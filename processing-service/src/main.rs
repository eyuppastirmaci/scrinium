use pdfium_render::prelude::*;
use processing_service::adapter::inbound::kafka::build_consumer;
use processing_service::adapter::outbound::messaging::KafkaEventPublisher;
use processing_service::adapter::outbound::persistence::SqlxProcessingJobRepository;
use processing_service::adapter::outbound::processing::digital_pdf_processor::DigitalPdfProcessor;
use processing_service::adapter::outbound::processing::image_processor::ImageProcessor;
use processing_service::adapter::outbound::processing::metadata::{
    CompositeMetadataExtractor, ImageExifMetadataExtractor, LanguageMetadataExtractor,
    PdfMetadataExtractor,
};
use processing_service::adapter::outbound::processing::preprocessing::*;
use processing_service::adapter::outbound::processing::scanned_pdf_processor::ScannedPdfProcessor;
use processing_service::adapter::outbound::processing::tesseract_ocr::TesseractOcr;
use processing_service::adapter::outbound::processing::thumbnail::{
    CompositeThumbnailGenerator, ImageThumbnailGenerator, PdfThumbnailGenerator,
};
use processing_service::adapter::outbound::storage::S3DocumentStorage;
use processing_service::application::{HandleError, ProcessDocument};
use processing_service::config::AppConfig;
use processing_service::domain;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

fn build_preprocessing_pipeline() -> PreprocessingPipeline {
    PreprocessingPipeline::new(vec![Box::new(GrayscaleStep), Box::new(DeskewStep)])
}

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

    let s3_config = aws_config::from_env()
        .endpoint_url(&config.storage_endpoint)
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            &config.storage_access_key,
            &config.storage_secret_key,
            None,
            None,
            "env",
        ))
        .region(aws_sdk_s3::config::Region::new("us-east-1"))
        .load()
        .await;

    let s3_client = aws_sdk_s3::Client::from_conf(
        aws_sdk_s3::config::Builder::from(&s3_config)
            .force_path_style(true)
            .build(),
    );

    let ocr: Arc<dyn domain::port::OcrEngine> = Arc::new(TesseractOcr::new(
        config.tesseract_path.clone(),
        config.tesseract_languages.clone(),
    ));
    println!(
        "processing-service Tesseract configured: {} ({})",
        config.tesseract_path, config.tesseract_languages
    );

    let digital_pdf_processor = DigitalPdfProcessor::new();
    let image_processor = ImageProcessor::new(build_preprocessing_pipeline(), Arc::clone(&ocr));

    let pdfium_bindings =
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&config.pdfium_path))
            .or_else(|_| Pdfium::bind_to_system_library());

    let (scanned_pdf_processor, pdf_thumbnail_generator) = match pdfium_bindings {
        Ok(bindings) => {
            println!("processing-service PDFium loaded");
            let pdfium = Arc::new(Pdfium::new(bindings));
            (
                Some(ScannedPdfProcessor::new(
                    Arc::clone(&pdfium),
                    build_preprocessing_pipeline(),
                    Arc::clone(&ocr),
                )),
                Some(PdfThumbnailGenerator::new(pdfium)),
            )
        }
        Err(e) => {
            eprintln!("PDFium not available, scanned PDF processing disabled: {e}");
            (None, None)
        }
    };

    let thumbnail_generator = CompositeThumbnailGenerator::new(
        Box::new(ImageThumbnailGenerator::new()),
        pdf_thumbnail_generator.map(|g| Box::new(g) as Box<dyn domain::port::ThumbnailGenerator>),
    );

    let publisher = KafkaEventPublisher::new(&config.kafka_brokers);
    let repository = SqlxProcessingJobRepository::new(db_pool);
    let storage = S3DocumentStorage::new(s3_client, config.storage_bucket);
    let metadata_extractor = CompositeMetadataExtractor::new(vec![
        Box::new(PdfMetadataExtractor::new()),
        Box::new(ImageExifMetadataExtractor::new()),
        Box::new(LanguageMetadataExtractor::new()),
    ]);

    let mut use_case = ProcessDocument::new(&publisher, &repository, &storage)
        .with_digital_pdf_processor(Box::new(digital_pdf_processor))
        .with_image_processor(Box::new(image_processor))
        .with_metadata_extractor(Box::new(metadata_extractor))
        .with_thumbnail_generator(Box::new(thumbnail_generator));

    if let Some(processor) = scanned_pdf_processor {
        use_case = use_case.with_scanned_pdf_processor(Box::new(processor));
    }

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
