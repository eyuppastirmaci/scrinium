use image::DynamicImage;

#[derive(Debug)]
pub struct PreprocessingError(pub String);

pub trait PreprocessingStep: Send + Sync {
    fn name(&self) -> &str;
    fn apply(&self, image: DynamicImage) -> Result<DynamicImage, PreprocessingError>;
}
