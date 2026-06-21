use crate::domain::port::{PreprocessingError, PreprocessingStep};
use image::DynamicImage;
use image::imageops::FilterType;

const TARGET_DPI: f64 = 300.0;
const MIN_TEXT_LINE_HEIGHT: u32 = 20;

pub struct DpiNormalizationStep;

impl PreprocessingStep for DpiNormalizationStep {
    fn name(&self) -> &str {
        "dpi_normalization"
    }

    fn apply(&self, image: DynamicImage) -> Result<DynamicImage, PreprocessingError> {
        let (w, h) = (image.width(), image.height());

        if h < MIN_TEXT_LINE_HEIGHT * 10 {
            let scale = TARGET_DPI / 72.0;
            let new_w = (w as f64 * scale) as u32;
            let new_h = (h as f64 * scale) as u32;
            Ok(image.resize_exact(new_w, new_h, FilterType::Lanczos3))
        } else {
            Ok(image)
        }
    }
}
