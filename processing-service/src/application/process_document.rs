use crate::adapter::inbound::kafka::contract::DocumentUploaded;
use crate::adapter::outbound::processing::pdf_detector::{self, PdfType};
use crate::domain::model::{
    ExtractedDocumentMetadata, NewDocumentMetadata, NewProcessingJob, ProcessingJobStatus,
    ProcessingResult,
};
use crate::domain::model::ThumbnailSize;
use crate::domain::port::{
    DocumentProcessor, DocumentStorage, EventPublisher, MetadataExtractionInput, MetadataExtractor,
    MetadataRepository, ProcessingJobRepository, ThumbnailGenerator, ThumbnailRepository,
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
    metadata_repository: Option<&'a dyn MetadataRepository>,
    metadata_extractor: Option<Box<dyn MetadataExtractor>>,
    thumbnail_generator: Option<Box<dyn ThumbnailGenerator>>,
    thumbnail_repository: Option<&'a dyn ThumbnailRepository>,
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
            metadata_repository: None,
            metadata_extractor: None,
            thumbnail_generator: None,
            thumbnail_repository: None,
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

    pub fn with_metadata(
        mut self,
        repository: &'a dyn MetadataRepository,
        extractor: Box<dyn MetadataExtractor>,
    ) -> Self {
        self.metadata_repository = Some(repository);
        self.metadata_extractor = Some(extractor);
        self
    }

    pub fn with_thumbnail_generator(
        mut self,
        generator: Box<dyn ThumbnailGenerator>,
        repository: &'a dyn ThumbnailRepository,
    ) -> Self {
        self.thumbnail_generator = Some(generator);
        self.thumbnail_repository = Some(repository);
        self
    }

    pub async fn handle(&self, raw_payload: &[u8]) -> Result<(), HandleError> {
        let uploaded: DocumentUploaded = serde_json::from_slice(raw_payload)
            .map_err(|e| HandleError::Skip(format!("malformed event: {e}")))?;

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

        let content = self
            .storage
            .read_document(&payload.storage_object_key)
            .await
            .map_err(|e| HandleError::Retry(e.0))?;

        println!(
            "read {} bytes from storage for document {document_id}",
            content.len()
        );

        let result = self
            .process_content(&content, &payload.content_type, document_id)
            .await;

        match result {
            Ok(Some(r)) => {
                let total_chars: usize = r.pages.iter().map(|p| p.text.len()).sum();
                println!(
                    "processed {} pages, {} chars for document {document_id}",
                    r.pages.len(),
                    total_chars
                );

                self.repository
                    .save_extracted_pages(document_id, &r.pages)
                    .await
                    .map_err(|e| HandleError::Retry(e.0))?;
                println!(
                    "saved {} extracted pages for document {document_id}",
                    r.pages.len()
                );

                self.store_metadata(document_id, &content, &payload.content_type, Some(&r))
                    .await?;

                self.generate_thumbnails(document_id, &content, &payload.content_type)
                    .await;

                self.repository
                    .mark_completed(document_id)
                    .await
                    .map_err(|e| HandleError::Retry(e.0))?;

                self.publisher
                    .processing_completed(&document_id.to_string())
                    .await
                    .map_err(|e| HandleError::Retry(e.0))?;

                println!("published document.processing.completed for {document_id}");
            }
            Ok(None) => {
                self.store_metadata(document_id, &content, &payload.content_type, None)
                    .await?;

                self.generate_thumbnails(document_id, &content, &payload.content_type)
                    .await;

                self.repository
                    .mark_completed(document_id)
                    .await
                    .map_err(|e| HandleError::Retry(e.0))?;

                self.publisher
                    .processing_completed(&document_id.to_string())
                    .await
                    .map_err(|e| HandleError::Retry(e.0))?;

                println!("no extraction performed, marked completed for document {document_id}");
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

                println!("published document.processing.failed for {document_id}");
            }
        }

        Ok(())
    }

    async fn store_metadata(
        &self,
        document_id: Uuid,
        content: &[u8],
        content_type: &str,
        result: Option<&ProcessingResult>,
    ) -> Result<(), HandleError> {
        let (Some(repository), Some(extractor)) =
            (self.metadata_repository, self.metadata_extractor.as_ref())
        else {
            return Ok(());
        };

        let extracted_text = result.map(extracted_text_for_metadata);
        let extracted = match extractor
            .extract(MetadataExtractionInput {
                content,
                content_type,
                extracted_text: extracted_text.as_deref(),
            })
            .await
        {
            Ok(metadata) => metadata,
            Err(e) => {
                eprintln!(
                    "metadata extraction failed for document {document_id}: {}",
                    e.0
                );
                ExtractedDocumentMetadata::default()
            }
        };

        let metadata_json =
            serde_json::to_string(&extracted.metadata_json).unwrap_or_else(|_| "{}".to_string());

        repository
            .upsert(NewDocumentMetadata {
                document_id,
                page_count: extracted.page_count,
                pdf_created_at: extracted.pdf_created_at,
                pdf_modified_at: extracted.pdf_modified_at,
                pdf_author: extracted.pdf_author,
                pdf_title: extracted.pdf_title,
                image_captured_at: extracted.image_captured_at,
                image_device: extracted.image_device,
                image_gps_present: extracted.image_gps_present,
                image_gps_redacted: extracted.image_gps_redacted,
                detected_language: extracted.detected_language,
                metadata_json,
            })
            .await
            .map_err(|e| HandleError::Retry(e.0))?;

        println!("saved metadata for document {document_id}");

        Ok(())
    }

    async fn generate_thumbnails(
        &self,
        document_id: Uuid,
        content: &[u8],
        content_type: &str,
    ) {
        let Some(generator) = &self.thumbnail_generator else {
            return;
        };

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
                    if let Some(repo) = self.thumbnail_repository {
                        if let Err(e) = repo
                            .upsert(document_id, size, &key, thumb.width as i32, thumb.height as i32)
                            .await
                        {
                            eprintln!(
                                "failed to persist {} thumbnail record for document {document_id}: {}",
                                size.suffix(), e.0
                            );
                            continue;
                        }
                    }
                    println!(
                        "saved {} thumbnail for document {document_id} ({}x{}, {} bytes)",
                        size.suffix(), thumb.width, thumb.height, thumb.bytes.len()
                    );
                }
                Err(e) => eprintln!(
                    "failed to generate {} thumbnail for document {document_id}: {}",
                    size.suffix(), e.0
                ),
            }
        }
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
                    if let Some(processor) = &self.scanned_pdf_processor {
                        let result = processor.process(content).await.map_err(|e| e.0)?;
                        return Ok(Some(result));
                    } else {
                        println!("  no scanned PDF processor available, skipping");
                    }
                }
            }
        } else if content_type.starts_with("image/") {
            println!("document {document_id}: image, preprocessing + OCR");
            if let Some(processor) = &self.image_processor {
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

fn extracted_text_for_metadata(result: &ProcessingResult) -> String {
    result
        .pages
        .iter()
        .map(|page| page.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug)]
pub enum HandleError {
    Skip(String),
    Retry(String),
}
