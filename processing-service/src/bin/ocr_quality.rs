use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use image::ImageFormat;
use imageproc::contrast::{ThresholdType, otsu_level, threshold};
use processing_service::adapter::outbound::processing::preprocessing::{
    DenoiseStep, DeskewStep, DpiNormalizationStep, GrayscaleStep,
};
use processing_service::adapter::outbound::processing::tesseract_ocr::TesseractOcr;
use processing_service::config::AppConfig;
use processing_service::domain::port::{OcrEngine, PreprocessingStep};

const DEFAULT_FIXTURES: &str = "fixtures/ocr-quality/external/sroie/manifest.toml";
const DEFAULT_VARIANT: &str = "raw-grayscale";
const DESKEW_VARIANT: &str = "grayscale-deskew";
const DENOISE_VARIANT: &str = "grayscale-denoise";
const DPI_VARIANT: &str = "grayscale-dpi";
const BINARIZE_VARIANT: &str = "grayscale-binarize";

#[derive(Debug)]
struct CliArgs {
    fixtures: PathBuf,
    variant: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    match parse_args(std::env::args().skip(1)) {
        Ok(Some(args)) => match run(args).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(1)
            }
        },
        Ok(None) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            print_usage();
            ExitCode::from(2)
        }
    }
}

async fn run(args: CliArgs) -> Result<(), String> {
    let manifest = load_manifest(&args.fixtures)?;
    let config = AppConfig::from_env();
    let language = manifest.language.clone();
    let ocr = TesseractOcr::new(config.tesseract_path.clone(), language.clone());

    println!("OCR quality harness");
    println!("  fixtures: {}", args.fixtures.display());
    println!("  variant: {}", args.variant);
    println!("  source: {}", manifest.source_name);
    println!("  language: {}", language);
    println!("  tesseract: {}", config.tesseract_path);
    println!("  loaded fixtures: {}", manifest.fixtures.len());

    let mut results = Vec::with_capacity(manifest.fixtures.len());

    for fixture in &manifest.fixtures {
        let result = evaluate_fixture(fixture, &args.variant, &ocr).await?;
        print_fixture_result(&result);
        results.push(result);
    }

    print_summary(&results);

    Ok(())
}

#[derive(Debug)]
struct FixtureResult {
    id: String,
    cer: f64,
    wer: f64,
    ground_truth_chars: usize,
    prediction_chars: usize,
}

async fn evaluate_fixture(
    fixture: &ResolvedFixture,
    variant: &str,
    ocr: &dyn OcrEngine,
) -> Result<FixtureResult, String> {
    let ground_truth = fs::read_to_string(&fixture.ground_truth).map_err(|e| {
        format!(
            "failed to read ground truth for {} at {}: {e}",
            fixture.id,
            fixture.ground_truth.display()
        )
    })?;
    let image_path = preprocess_fixture_image(fixture, variant)?;
    let prediction_result = ocr.recognize(&image_path).await;
    let _ = fs::remove_file(&image_path);
    let prediction =
        prediction_result.map_err(|e| format!("OCR failed for {}: {}", fixture.id, e.0))?;
    let rates = calculate_error_rates(&ground_truth, &prediction);

    Ok(FixtureResult {
        id: fixture.id.clone(),
        cer: rates.cer,
        wer: rates.wer,
        ground_truth_chars: normalize_text(&ground_truth).chars().count(),
        prediction_chars: normalize_text(&prediction).chars().count(),
    })
}

fn preprocess_fixture_image(fixture: &ResolvedFixture, variant: &str) -> Result<PathBuf, String> {
    let image = image::open(&fixture.input).map_err(|e| {
        format!(
            "failed to open fixture image {} at {}: {e}",
            fixture.id,
            fixture.input.display()
        )
    })?;
    let preprocessed = GrayscaleStep
        .apply(image)
        .map_err(|e| format!("preprocessing failed for {}: {}", fixture.id, e.0))?;
    let preprocessed = match variant {
        DEFAULT_VARIANT => preprocessed,
        DESKEW_VARIANT => DeskewStep
            .apply(preprocessed)
            .map_err(|e| format!("deskew failed for {}: {}", fixture.id, e.0))?,
        DENOISE_VARIANT => DenoiseStep
            .apply(preprocessed)
            .map_err(|e| format!("denoise failed for {}: {}", fixture.id, e.0))?,
        DPI_VARIANT => DpiNormalizationStep
            .apply(preprocessed)
            .map_err(|e| format!("DPI normalization failed for {}: {}", fixture.id, e.0))?,
        BINARIZE_VARIANT => binarize_with_otsu(preprocessed),
        _ => {
            return Err(format!(
                "unsupported OCR quality variant '{variant}'. Currently implemented: {DEFAULT_VARIANT}, {DESKEW_VARIANT}, {DENOISE_VARIANT}, {DPI_VARIANT}, {BINARIZE_VARIANT}"
            ));
        }
    };
    let temp_path = std::env::temp_dir().join(format!("scrinium_ocr_quality_{}.png", fixture.id));

    preprocessed
        .save_with_format(&temp_path, ImageFormat::Png)
        .map_err(|e| {
            format!(
                "failed to save preprocessed image for {} at {}: {e}",
                fixture.id,
                temp_path.display()
            )
        })?;

    Ok(temp_path)
}

fn binarize_with_otsu(image: image::DynamicImage) -> image::DynamicImage {
    let gray = image.to_luma8();
    let level = otsu_level(&gray);
    image::DynamicImage::ImageLuma8(threshold(&gray, level, ThresholdType::Binary))
}

fn print_fixture_result(result: &FixtureResult) {
    println!(
        "  {:<12} CER {:>7.2}%  WER {:>7.2}%  chars truth/pred {:>5}/{:<5}",
        result.id,
        result.cer * 100.0,
        result.wer * 100.0,
        result.ground_truth_chars,
        result.prediction_chars
    );
}

fn print_summary(results: &[FixtureResult]) {
    if results.is_empty() {
        return;
    }

    let avg_cer = results.iter().map(|r| r.cer).sum::<f64>() / results.len() as f64;
    let avg_wer = results.iter().map(|r| r.wer).sum::<f64>() / results.len() as f64;

    println!();
    println!("Summary");
    println!("  fixtures: {}", results.len());
    println!("  average CER: {:.2}%", avg_cer * 100.0);
    println!("  average WER: {:.2}%", avg_wer * 100.0);
}

#[allow(dead_code)]
fn print_manifest_preview(manifest: &QualityManifest) {
    for fixture in manifest.fixtures.iter().take(3) {
        println!();
        println!("  {}", fixture.id);
        println!("    kind: {}", fixture.kind);
        println!("    input: {}", fixture.input.display());
        println!("    ground truth: {}", fixture.ground_truth.display());
    }
    if manifest.fixtures.len() > 3 {
        println!();
        println!("  ... {} more fixtures", manifest.fixtures.len() - 3);
    }
}

#[derive(Debug)]
struct QualityManifest {
    source_name: String,
    language: String,
    fixtures: Vec<ResolvedFixture>,
}

#[derive(Debug)]
struct ResolvedFixture {
    id: String,
    kind: String,
    input: PathBuf,
    ground_truth: PathBuf,
}

#[derive(Debug, Deserialize)]
struct RawManifest {
    source_name: String,
    language: String,
    fixtures: Vec<RawFixture>,
}

#[derive(Debug, Deserialize)]
struct RawFixture {
    id: String,
    kind: String,
    input: PathBuf,
    ground_truth: PathBuf,
}

fn load_manifest(path: &Path) -> Result<QualityManifest, String> {
    if !path.exists() {
        return Err(format!(
            "OCR quality fixture manifest does not exist: {}",
            path.display()
        ));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read fixture manifest {}: {e}", path.display()))?;
    let raw: RawManifest = toml::from_str(&content)
        .map_err(|e| format!("failed to parse fixture manifest {}: {e}", path.display()))?;

    if raw.fixtures.is_empty() {
        return Err(format!(
            "fixture manifest has no fixtures: {}",
            path.display()
        ));
    }

    let base_dir = path.parent().ok_or_else(|| {
        format!(
            "fixture manifest has no parent directory: {}",
            path.display()
        )
    })?;
    let fixtures = raw
        .fixtures
        .into_iter()
        .map(|fixture| resolve_fixture(base_dir, fixture))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(QualityManifest {
        source_name: raw.source_name,
        language: raw.language,
        fixtures,
    })
}

fn resolve_fixture(base_dir: &Path, fixture: RawFixture) -> Result<ResolvedFixture, String> {
    let input = base_dir.join(&fixture.input);
    let ground_truth = base_dir.join(&fixture.ground_truth);

    if !input.exists() {
        return Err(format!(
            "fixture {} input does not exist: {}",
            fixture.id,
            input.display()
        ));
    }
    if !ground_truth.exists() {
        return Err(format!(
            "fixture {} ground truth does not exist: {}",
            fixture.id,
            ground_truth.display()
        ));
    }

    Ok(ResolvedFixture {
        id: fixture.id,
        kind: fixture.kind,
        input,
        ground_truth,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ErrorRates {
    cer: f64,
    wer: f64,
}

fn calculate_error_rates(ground_truth: &str, prediction: &str) -> ErrorRates {
    let normalized_ground_truth = normalize_text(ground_truth);
    let normalized_prediction = normalize_text(prediction);

    ErrorRates {
        cer: character_error_rate(&normalized_ground_truth, &normalized_prediction),
        wer: word_error_rate(&normalized_ground_truth, &normalized_prediction),
    }
}

fn character_error_rate(ground_truth: &str, prediction: &str) -> f64 {
    let expected: Vec<char> = ground_truth.chars().collect();
    let actual: Vec<char> = prediction.chars().collect();
    normalized_distance(&expected, &actual)
}

fn word_error_rate(ground_truth: &str, prediction: &str) -> f64 {
    let expected: Vec<&str> = ground_truth.split_whitespace().collect();
    let actual: Vec<&str> = prediction.split_whitespace().collect();
    normalized_distance(&expected, &actual)
}

fn normalized_distance<T: Eq>(expected: &[T], actual: &[T]) -> f64 {
    if expected.is_empty() {
        return if actual.is_empty() { 0.0 } else { 1.0 };
    }

    levenshtein_distance(expected, actual) as f64 / expected.len() as f64
}

fn levenshtein_distance<T: Eq>(expected: &[T], actual: &[T]) -> usize {
    if expected.is_empty() {
        return actual.len();
    }
    if actual.is_empty() {
        return expected.len();
    }

    let mut previous: Vec<usize> = (0..=actual.len()).collect();
    let mut current = vec![0; actual.len() + 1];

    for (i, expected_item) in expected.iter().enumerate() {
        current[0] = i + 1;

        for (j, actual_item) in actual.iter().enumerate() {
            let substitution_cost = if expected_item == actual_item { 0 } else { 1 };
            current[j + 1] = (previous[j + 1] + 1)
                .min(current[j] + 1)
                .min(previous[j] + substitution_cost);
        }

        std::mem::swap(&mut previous, &mut current);
    }

    previous[actual.len()]
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_args<I>(args: I) -> Result<Option<CliArgs>, String>
where
    I: IntoIterator<Item = String>,
{
    let mut fixtures = PathBuf::from(DEFAULT_FIXTURES);
    let mut variant = DEFAULT_VARIANT.to_string();

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                return Ok(None);
            }
            "--fixtures" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--fixtures requires a manifest path".to_string())?;
                fixtures = PathBuf::from(value);
            }
            "--variant" => {
                variant = iter
                    .next()
                    .ok_or_else(|| "--variant requires a variant name".to_string())?;
            }
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
    }

    Ok(Some(CliArgs { fixtures, variant }))
}

fn print_usage() {
    println!(
        "Usage: cargo run --bin ocr_quality -- [--fixtures <manifest.toml>] [--variant <name>]\n\
\n\
Defaults:\n\
  --fixtures {DEFAULT_FIXTURES}\n\
  --variant  {DEFAULT_VARIANT}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cer_is_zero_for_identical_text() {
        assert_eq!(character_error_rate("abc", "abc"), 0.0);
    }

    #[test]
    fn cer_counts_single_character_substitution() {
        assert_eq!(character_error_rate("abc", "axc"), 1.0 / 3.0);
    }

    #[test]
    fn cer_handles_turkish_unicode_characters() {
        assert_eq!(character_error_rate("ışık", "isik"), 3.0 / 4.0);
    }

    #[test]
    fn wer_is_zero_for_identical_words() {
        assert_eq!(word_error_rate("hello world", "hello world"), 0.0);
    }

    #[test]
    fn wer_counts_missing_word() {
        assert_eq!(
            word_error_rate("hello small world", "hello world"),
            1.0 / 3.0
        );
    }

    #[test]
    fn empty_ground_truth_with_prediction_is_full_error() {
        assert_eq!(character_error_rate("", "abc"), 1.0);
        assert_eq!(word_error_rate("", "abc"), 1.0);
    }

    #[test]
    fn text_normalization_collapses_whitespace() {
        let rates = calculate_error_rates("hello\n  world", "hello world");
        assert_eq!(rates, ErrorRates { cer: 0.0, wer: 0.0 });
    }
}
