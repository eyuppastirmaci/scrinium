use crate::adapter::outbound::processing::preprocessing::PreprocessingPipeline;
use crate::domain::model::{ExtractedPage, ProcessingResult};
use crate::domain::port::{DocumentProcessor, OcrEngine, ProcessingError};
use image::{DynamicImage, ImageFormat};
use pdfium_render::prelude::*;
use std::sync::Arc;

pub struct ScannedPdfProcessor {
    pdfium: Arc<Pdfium>,
    pipeline: PreprocessingPipeline,
    ocr: Arc<dyn OcrEngine>,
}

impl ScannedPdfProcessor {
    pub fn new(pdfium: Arc<Pdfium>, pipeline: PreprocessingPipeline, ocr: Arc<dyn OcrEngine>) -> Self {
        Self {
            pdfium,
            pipeline,
            ocr,
        }
    }

    fn render_and_preprocess(
        &self,
        content: &[u8],
    ) -> Result<Vec<(i32, DynamicImage)>, ProcessingError> {
        let doc = self
            .pdfium
            .load_pdf_from_byte_slice(content, None)
            .map_err(|e| ProcessingError(format!("failed to load PDF: {e}")))?;

        let page_count = doc.pages().len();
        let mut images = Vec::with_capacity(page_count as usize);

        for (i, page) in doc.pages().iter().enumerate() {
            println!("  rendering page {}/{page_count}", i + 1);

            let render_config = PdfRenderConfig::new()
                .set_target_width(2480)
                .set_maximum_height(3508);

            let bitmap = page
                .render_with_config(&render_config)
                .map_err(|e| ProcessingError(format!("render failed for page {}: {e}", i + 1)))?;

            let dynamic_image = bitmap
                .as_image()
                .as_rgba8()
                .map(|rgba| DynamicImage::ImageRgba8(rgba.clone()))
                .ok_or_else(|| {
                    ProcessingError(format!("bitmap conversion failed for page {}", i + 1))
                })?;

            let preprocessed = self.pipeline.run(dynamic_image).map_err(|e| {
                ProcessingError(format!("preprocessing failed for page {}: {}", i + 1, e.0))
            })?;

            println!(
                "  page {} preprocessed: {}x{}",
                i + 1,
                preprocessed.width(),
                preprocessed.height()
            );
            images.push(((i + 1) as i32, preprocessed));
        }

        Ok(images)
    }
}

#[async_trait::async_trait]
impl DocumentProcessor for ScannedPdfProcessor {
    async fn process(&self, content: &[u8]) -> Result<ProcessingResult, ProcessingError> {
        let images = self.render_and_preprocess(content)?;

        let mut pages = Vec::with_capacity(images.len());

        for (page_num, preprocessed) in images {
            let temp_path =
                std::env::temp_dir().join(format!("scrinium_ocr_{}.png", uuid::Uuid::new_v4()));
            preprocessed
                .save_with_format(&temp_path, ImageFormat::Png)
                .map_err(|e| ProcessingError(format!("failed to save temp image: {e}")))?;

            println!("  running OCR on page {page_num}...");
            let text =
                self.ocr.recognize(&temp_path).await.map_err(|e| {
                    ProcessingError(format!("OCR failed for page {page_num}: {}", e.0))
                })?;

            let _ = std::fs::remove_file(&temp_path);

            let trimmed = text.trim().to_string();
            println!("  page {page_num} OCR: {} chars", trimmed.len());

            pages.push(ExtractedPage {
                page_number: page_num,
                text: trimmed,
            });
        }

        Ok(ProcessingResult { pages })
    }
}
