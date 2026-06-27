use crate::domain::model::{ExtractedPage, ProcessingResult};
use crate::domain::port::{DocumentProcessor, ProcessingError};

pub struct DigitalPdfProcessor;

impl DigitalPdfProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DocumentProcessor for DigitalPdfProcessor {
    async fn process(&self, content: &[u8]) -> Result<ProcessingResult, ProcessingError> {
        let text = pdf_extract::extract_text_from_mem(content)
            .map_err(|e| ProcessingError(format!("PDF text extraction failed: {e}")))?;

        let page_count = lopdf::Document::load_mem(content)
            .map(|doc| doc.get_pages().len() as i32)
            .unwrap_or(1);

        let pages = vec![ExtractedPage {
            page_number: 1,
            text: text.trim().to_string(),
        }];

        println!(
            "digital PDF: {page_count} pages, extracted {} chars",
            pages[0].text.len()
        );

        Ok(ProcessingResult { pages })
    }
}
