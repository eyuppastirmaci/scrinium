use crate::adapter::outbound::processing::preprocessing::PreprocessingPipeline;
use crate::domain::model::{ExtractedPage, ProcessingResult};
use crate::domain::port::{DocumentProcessor, OcrEngine, ProcessingError};
use image::ImageFormat;
use std::sync::Arc;

pub struct ImageProcessor {
    pipeline: PreprocessingPipeline,
    ocr: Arc<dyn OcrEngine>,
}

impl ImageProcessor {
    pub fn new(pipeline: PreprocessingPipeline, ocr: Arc<dyn OcrEngine>) -> Self {
        Self { pipeline, ocr }
    }
}

#[async_trait::async_trait]
impl DocumentProcessor for ImageProcessor {
    async fn process(&self, content: &[u8]) -> Result<ProcessingResult, ProcessingError> {
        let img = image::load_from_memory(content)
            .map_err(|e| ProcessingError(format!("failed to load image: {e}")))?;

        println!("  image loaded: {}x{}", img.width(), img.height());

        let preprocessed = self
            .pipeline
            .run(img)
            .map_err(|e| ProcessingError(format!("preprocessing failed: {}", e.0)))?;

        println!(
            "  image preprocessed: {}x{}",
            preprocessed.width(),
            preprocessed.height()
        );

        let temp_path =
            std::env::temp_dir().join(format!("scrinium_ocr_{}.png", uuid::Uuid::new_v4()));
        preprocessed
            .save_with_format(&temp_path, ImageFormat::Png)
            .map_err(|e| ProcessingError(format!("failed to save temp image: {e}")))?;

        println!("  running OCR...");
        let text = self
            .ocr
            .recognize(&temp_path)
            .await
            .map_err(|e| ProcessingError(format!("OCR failed: {}", e.0)))?;

        let _ = std::fs::remove_file(&temp_path);

        let trimmed = text.trim().to_string();
        println!("  OCR result: {} chars", trimmed.len());

        Ok(ProcessingResult {
            pages: vec![ExtractedPage {
                page_number: 1,
                text: trimmed,
            }],
        })
    }
}
