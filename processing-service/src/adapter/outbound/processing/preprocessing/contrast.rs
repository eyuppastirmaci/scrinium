use crate::domain::port::{PreprocessingError, PreprocessingStep};
use image::DynamicImage;
use imageproc::contrast::stretch_contrast;

pub struct ContrastNormalizationStep;

impl PreprocessingStep for ContrastNormalizationStep {
    fn name(&self) -> &str {
        "contrast_normalization"
    }

    fn apply(&self, image: DynamicImage) -> Result<DynamicImage, PreprocessingError> {
        let gray = image.to_luma8();
        let stretched = stretch_contrast(&gray, 0, 255, 0, 255);
        Ok(DynamicImage::ImageLuma8(stretched))
    }
}
