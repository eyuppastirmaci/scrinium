use crate::adapter::inbound::kafka::contract::DocumentUploaded;
use crate::adapter::outbound::processing::pdf_detector::{self, PdfType};
use crate::domain::model::{
    ExtractedDocumentMetadata, NewProcessingJob, ProcessingCompletedEvent, ProcessingJobStatus,
    ProcessingResult, ThumbnailInfo, ThumbnailSize,
};
use crate::domain::port::{
    DocumentProcessor, DocumentStorage, EventPublisher, MetadataExtractionInput, MetadataExtractor,
    ProcessingJobRepository, ProgressReporter, ThumbnailGenerator,
};
use uuid::Uuid;

pub struct ProcessDocument<'a, P, R, S>
where
    P: EventPublisher,
    R: ProcessingJobRepository,
    S: DocumentStorage,
{
    publisher: &'a P,
    repository: &'a R,
    storage: &'a S,
    digital_pdf_processor: Option<Box<dyn DocumentProcessor>>,
    scanned_pdf_processor: Option<Box<dyn DocumentProcessor>>,
    image_processor: Option<Box<dyn DocumentProcessor>>,
    metadata_extractor: Option<Box<dyn MetadataExtractor>>,
    thumbnail_generator: Option<Box<dyn ThumbnailGenerator>>,
    progress_reporter: Option<Box<dyn ProgressReporter>>,
}

impl<'a, P, R, S> ProcessDocument<'a, P, R, S>
where
    P: EventPublisher,
    R: ProcessingJobRepository,
    S: DocumentStorage,
{
    pub fn new(publisher: &'a P, repository: &'a R, storage: &'a S) -> Self {
        Self {
            publisher,
            repository,
            storage,
            digital_pdf_processor: None,
            scanned_pdf_processor: None,
            image_processor: None,
            metadata_extractor: None,
            thumbnail_generator: None,
            progress_reporter: None,
        }
    }

    pub fn with_digital_pdf_processor(mut self, processor: Box<dyn DocumentProcessor>) -> Self {
        self.digital_pdf_processor = Some(processor);
        self
    }

    pub fn with_scanned_pdf_processor(mut self, processor: Box<dyn DocumentProcessor>) -> Self {
        self.scanned_pdf_processor = Some(processor);
        self
    }

    pub fn with_image_processor(mut self, processor: Box<dyn DocumentProcessor>) -> Self {
        self.image_processor = Some(processor);
        self
    }

    pub fn with_metadata_extractor(mut self, extractor: Box<dyn MetadataExtractor>) -> Self {
        self.metadata_extractor = Some(extractor);
        self
    }

    pub fn with_thumbnail_generator(mut self, generator: Box<dyn ThumbnailGenerator>) -> Self {
        self.thumbnail_generator = Some(generator);
        self
    }

    pub fn with_progress_reporter(mut self, reporter: Box<dyn ProgressReporter>) -> Self {
        self.progress_reporter = Some(reporter);
        self
    }

    pub async fn handle(&self, raw_payload: &[u8]) -> Result<(), HandleError> {
        let uploaded: DocumentUploaded = serde_json::from_slice(raw_payload)
            .map_err(|e| HandleError::Skip(format!("malformed event: {e}")))?;

        let upload_timestamp = uploaded.timestamp;
        let payload = uploaded.payload;
        let document_id: Uuid = payload
            .document_id
            .parse()
            .map_err(|e| HandleError::Skip(format!("invalid document_id: {e}")))?;

        println!(
            "received document.uploaded for {document_id}: {} ({} bytes, {})",
            payload.file_name, payload.size_bytes, payload.content_type
        );

        let existing = self
            .repository
            .find_by_document_id(document_id)
            .await
            .map_err(|e| HandleError::Retry(e.0))?;

        if let Some(job) = existing {
            if job.status == ProcessingJobStatus::Completed {
                println!("skipping already-completed document {document_id}");
                return Ok(());
            }
        }

        let new_job = NewProcessingJob {
            document_id,
            file_name: payload.file_name.clone(),
            content_type: payload.content_type.clone(),
            size_bytes: payload.size_bytes,
            storage_object_key: payload.storage_object_key.clone(),
            sha256: payload.sha256,
        };

        self.repository
            .start_or_update_received(new_job)
            .await
            .map_err(|e| HandleError::Retry(e.0))?;

        self.report_progress(document_id, "received", 0).await;

        let content = self
            .storage
            .read_document(&payload.storage_object_key)
            .await
            .map_err(|e| HandleError::Retry(e.0))?;

        println!(
            "read {} bytes from storage for document {document_id}",
            content.len()
        );

        self.report_progress(document_id, "extracting_text", 20).await;

        let result = self
            .process_content(&content, &payload.content_type, document_id)
            .await;

        match result {
            Ok(pages) => {
                let pages = pages.map(|r| r.pages).unwrap_or_default();
                let total_chars: usize = pages.iter().map(|p| p.text.len()).sum();
                println!(
                    "processed {} pages, {} chars for document {document_id}",
                    pages.len(),
                    total_chars
                );

                self.report_progress(document_id, "extracting_metadata", 70).await;

                let metadata = self
                    .extract_metadata(document_id, &content, &payload.content_type, &pages)
                    .await;

                self.report_progress(document_id, "generating_thumbnail", 85).await;

                let thumbnails = self
                    .generate_thumbnails(document_id, &content, &payload.content_type)
                    .await;

                self.repository
                    .mark_completed(document_id)
                    .await
                    .map_err(|e| HandleError::Retry(e.0))?;

                let event = ProcessingCompletedEvent {
                    document_id,
                    file_name: payload.file_name.clone(),
                    content_type: payload.content_type.clone(),
                    created_at: upload_timestamp.clone(),
                    pages,
                    metadata,
                    thumbnails,
                };

                self.publisher
                    .processing_completed(&event)
                    .await
                    .map_err(|e| HandleError::Retry(e.0))?;

                self.report_progress(document_id, "completed", 100).await;

                println!("published document.processing.completed for {document_id}");
            }
            Err(reason) => {
                eprintln!("processing failed for document {document_id}: {reason}");

                self.repository
                    .mark_failed(document_id, &reason)
                    .await
                    .map_err(|e| HandleError::Retry(e.0))?;

                self.publisher
                    .processing_failed(&document_id.to_string(), &reason)
                    .await
                    .map_err(|e| HandleError::Retry(e.0))?;

                self.report_progress(document_id, "failed", -1).await;

                println!("published document.processing.failed for {document_id}");
            }
        }

        Ok(())
    }

    async fn report_progress(&self, document_id: Uuid, step: &str, progress: i32) {
        if let Some(reporter) = &self.progress_reporter {
            if let Err(e) = reporter.report(document_id, step, progress).await {
                eprintln!("progress report failed for {document_id}: {}", e.0);
            }
        }
    }

    async fn extract_metadata(
        &self,
        document_id: Uuid,
        content: &[u8],
        content_type: &str,
        pages: &[crate::domain::model::ExtractedPage],
    ) -> ExtractedDocumentMetadata {
        let Some(extractor) = self.metadata_extractor.as_ref() else {
            return ExtractedDocumentMetadata::default();
        };

        let extracted_text = if pages.is_empty() {
            None
        } else {
            Some(
                pages
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        };

        match extractor
            .extract(MetadataExtractionInput {
                content,
                content_type,
                extracted_text: extracted_text.as_deref(),
            })
            .await
        {
            Ok(metadata) => {
                println!("extracted metadata for document {document_id}");
                metadata
            }
            Err(e) => {
                eprintln!(
                    "metadata extraction failed for document {document_id}: {}",
                    e.0
                );
                ExtractedDocumentMetadata::default()
            }
        }
    }

    async fn generate_thumbnails(
        &self,
        document_id: Uuid,
        content: &[u8],
        content_type: &str,
    ) -> Vec<ThumbnailInfo> {
        let Some(generator) = &self.thumbnail_generator else {
            return Vec::new();
        };

        let mut thumbnails = Vec::new();

        for &size in ThumbnailSize::all() {
            match generator.generate(content, content_type, size) {
                Ok(thumb) => {
                    let key = ThumbnailSize::storage_key(document_id, size);
                    if let Err(e) = self.storage.write_object(&key, &thumb.bytes, "image/jpeg").await {
                        eprintln!(
                            "failed to store {} thumbnail for document {document_id}: {}",
                            size.suffix(), e.0
                        );
                        continue;
                    }
                    println!(
                        "saved {} thumbnail for document {document_id} ({}x{}, {} bytes)",
                        size.suffix(), thumb.width, thumb.height, thumb.bytes.len()
                    );
                    thumbnails.push(ThumbnailInfo {
                        size,
                        storage_key: key,
                        width: thumb.width,
                        height: thumb.height,
                    });
                }
                Err(e) => eprintln!(
                    "failed to generate {} thumbnail for document {document_id}: {}",
                    size.suffix(), e.0
                ),
            }
        }

        thumbnails
    }

    async fn process_content(
        &self,
        content: &[u8],
        content_type: &str,
        document_id: Uuid,
    ) -> Result<Option<ProcessingResult>, String> {
        if content_type == "application/pdf" {
            match pdf_detector::detect(content) {
                PdfType::Invalid(reason) => {
                    return Err(reason);
                }
                PdfType::Digital => {
                    println!("document {document_id}: digital PDF, extracting text directly");
                    if let Some(processor) = &self.digital_pdf_processor {
                        let result = processor.process(content).await.map_err(|e| e.0)?;
                        return Ok(Some(result));
                    }
                }
                PdfType::Scanned => {
                    println!(
                        "document {document_id}: scanned PDF, rendering + preprocessing + OCR"
                    );
                    self.report_progress(document_id, "preprocessing_image", 30).await;
                    if let Some(processor) = &self.scanned_pdf_processor {
                        self.report_progress(document_id, "running_ocr", 50).await;
                        let result = processor.process(content).await.map_err(|e| e.0)?;
                        return Ok(Some(result));
                    } else {
                        println!("  no scanned PDF processor available, skipping");
                    }
                }
            }
        } else if content_type.starts_with("image/") {
            println!("document {document_id}: image, preprocessing + OCR");
            self.report_progress(document_id, "preprocessing_image", 30).await;
            if let Some(processor) = &self.image_processor {
                self.report_progress(document_id, "running_ocr", 50).await;
                let result = processor.process(content).await.map_err(|e| e.0)?;
                return Ok(Some(result));
            } else {
                println!("  no image processor available");
            }
        } else {
            return Err(format!("unsupported content type: {content_type}"));
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub enum HandleError {
    Skip(String),
    Retry(String),
}
