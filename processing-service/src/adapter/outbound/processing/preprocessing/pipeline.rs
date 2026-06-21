use crate::domain::port::{PreprocessingError, PreprocessingStep};
use image::DynamicImage;

pub struct PreprocessingPipeline {
    steps: Vec<Box<dyn PreprocessingStep>>,
}

impl PreprocessingPipeline {
    pub fn new(steps: Vec<Box<dyn PreprocessingStep>>) -> Self {
        Self { steps }
    }

    pub fn run(&self, image: DynamicImage) -> Result<DynamicImage, PreprocessingError> {
        let mut current = image;
        for step in &self.steps {
            println!("    preprocessing: {}", step.name());
            current = step.apply(current)?;
        }
        Ok(current)
    }
}
