use crate::domain::port::{PreprocessingError, PreprocessingStep};
use image::DynamicImage;

pub struct GrayscaleStep;

impl PreprocessingStep for GrayscaleStep {
    fn name(&self) -> &str {
        "grayscale"
    }

    fn apply(&self, image: DynamicImage) -> Result<DynamicImage, PreprocessingError> {
        Ok(DynamicImage::ImageLuma8(image.to_luma8()))
    }
}
