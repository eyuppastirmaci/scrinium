use crate::domain::port::{PreprocessingError, PreprocessingStep};
use image::imageops::FilterType;
use image::{DynamicImage, Luma};
use imageproc::geometric_transformations::{Interpolation, rotate_about_center};

const MAX_ANALYSIS_DIMENSION: u32 = 1000;
const MAX_SKEW_DEGREES: i32 = 5;
const ANGLE_STEP_DEGREES: f32 = 0.5;
const MIN_CORRECTION_DEGREES: f32 = 0.25;
const MIN_SCORE_IMPROVEMENT: f64 = 0.03;
const FOREGROUND_THRESHOLD: u8 = 200;
const WHITE: Luma<u8> = Luma([255]);

pub struct DeskewStep;

impl PreprocessingStep for DeskewStep {
    fn name(&self) -> &str {
        "deskew"
    }

    fn apply(&self, image: DynamicImage) -> Result<DynamicImage, PreprocessingError> {
        let gray = image.to_luma8();
        let analysis_image = resize_for_analysis(&gray);
        let baseline_score = projection_score(&analysis_image);

        if baseline_score == 0.0 {
            return Ok(DynamicImage::ImageLuma8(gray));
        }

        let (best_angle, best_score) = find_best_angle(&analysis_image);
        let improvement = (best_score - baseline_score) / baseline_score;

        if best_angle.abs() < MIN_CORRECTION_DEGREES || improvement < MIN_SCORE_IMPROVEMENT {
            return Ok(DynamicImage::ImageLuma8(gray));
        }

        let rotated = rotate_about_center(
            &gray,
            best_angle.to_radians(),
            Interpolation::Bilinear,
            WHITE,
        );
        Ok(DynamicImage::ImageLuma8(rotated))
    }
}

fn resize_for_analysis(
    image: &image::ImageBuffer<Luma<u8>, Vec<u8>>,
) -> image::ImageBuffer<Luma<u8>, Vec<u8>> {
    let (width, height) = image.dimensions();
    let max_dimension = width.max(height);

    if max_dimension <= MAX_ANALYSIS_DIMENSION {
        return image.clone();
    }

    let scale = MAX_ANALYSIS_DIMENSION as f32 / max_dimension as f32;
    let resized_width = ((width as f32 * scale).round() as u32).max(1);
    let resized_height = ((height as f32 * scale).round() as u32).max(1);

    image::imageops::resize(image, resized_width, resized_height, FilterType::Triangle)
}

fn find_best_angle(image: &image::ImageBuffer<Luma<u8>, Vec<u8>>) -> (f32, f64) {
    let mut best_angle = 0.0;
    let mut best_score = projection_score(image);
    let steps_each_side = (MAX_SKEW_DEGREES as f32 / ANGLE_STEP_DEGREES) as i32;

    for step in -steps_each_side..=steps_each_side {
        let angle = step as f32 * ANGLE_STEP_DEGREES;
        if angle == 0.0 {
            continue;
        }

        let rotated = rotate_about_center(image, angle.to_radians(), Interpolation::Nearest, WHITE);
        let score = projection_score(&rotated);
        if score > best_score {
            best_score = score;
            best_angle = angle;
        }
    }

    (best_angle, best_score)
}

fn projection_score(image: &image::ImageBuffer<Luma<u8>, Vec<u8>>) -> f64 {
    let mut score = 0.0;

    for y in 0..image.height() {
        let mut row_foreground = 0u32;
        for x in 0..image.width() {
            if image.get_pixel(x, y)[0] < FOREGROUND_THRESHOLD {
                row_foreground += 1;
            }
        }
        score += f64::from(row_foreground * row_foreground);
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageBuffer;

    #[test]
    fn blank_image_is_left_unchanged() {
        let image = DynamicImage::ImageLuma8(ImageBuffer::from_pixel(80, 40, WHITE));
        let result = DeskewStep.apply(image).expect("deskew should succeed");

        assert_eq!(result.width(), 80);
        assert_eq!(result.height(), 40);
    }
}
