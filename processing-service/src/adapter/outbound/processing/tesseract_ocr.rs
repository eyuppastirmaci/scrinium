use crate::domain::port::{OcrEngine, OcrError};
use std::path::Path;
use tokio::process::Command;

pub struct TesseractOcr {
    binary_path: String,
    languages: String,
}

impl TesseractOcr {
    pub fn new(binary_path: String, languages: String) -> Self {
        Self {
            binary_path,
            languages,
        }
    }
}

#[async_trait::async_trait]
impl OcrEngine for TesseractOcr {
    async fn recognize(&self, image_path: &Path) -> Result<String, OcrError> {
        let output = Command::new(&self.binary_path)
            .arg(image_path)
            .arg("stdout")
            .arg("-l")
            .arg(&self.languages)
            .arg("--oem")
            .arg("1")
            .output()
            .await
            .map_err(|e| OcrError(format!("failed to run tesseract: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OcrError(format!("tesseract failed: {stderr}")));
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(text)
    }
}
