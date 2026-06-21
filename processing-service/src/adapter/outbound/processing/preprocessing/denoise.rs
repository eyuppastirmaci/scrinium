use crate::domain::port::{PreprocessingError, PreprocessingStep};
use image::DynamicImage;
use imageproc::filter::median_filter;

pub struct DenoiseStep;

impl PreprocessingStep for DenoiseStep {
    fn name(&self) -> &str {
        "denoise"
    }

    fn apply(&self, image: DynamicImage) -> Result<DynamicImage, PreprocessingError> {
        let gray = image.to_luma8();
        let filtered = median_filter(&gray, 1, 1);
        Ok(DynamicImage::ImageLuma8(filtered))
    }
}
