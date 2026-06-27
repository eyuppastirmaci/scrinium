# SROIE Grayscale + Binarization OCR Measurement

Command:

```powershell
cargo run --bin ocr_quality -- --fixtures fixtures/ocr-quality/external/sroie/manifest.toml --variant grayscale-binarize
```

Configuration:

- Variant: `grayscale-binarize`
- Binarization: Otsu threshold, binary output
- Language: `eng`
- Tesseract: local Tesseract executable from `PROCESSING_TESSERACT_PATH`
- Fixtures: 10

| Fixture | CER | WER | Truth Chars | OCR Chars |
| --- | ---: | ---: | ---: | ---: |
| `sroie_000` | 48.66% | 89.41% | 485 | 441 |
| `sroie_001` | 55.26% | 82.35% | 684 | 480 |
| `sroie_002` | 15.49% | 48.78% | 723 | 751 |
| `sroie_003` | 52.91% | 83.81% | 584 | 485 |
| `sroie_004` | 47.18% | 81.25% | 797 | 685 |
| `sroie_005` | 48.13% | 74.19% | 374 | 353 |
| `sroie_006` | 33.41% | 56.69% | 916 | 867 |
| `sroie_007` | 24.04% | 60.27% | 441 | 451 |
| `sroie_008` | 30.86% | 61.78% | 891 | 828 |
| `sroie_009` | 59.44% | 80.99% | 784 | 781 |

Summary:

- Average CER: 41.54%
- Average WER: 71.95%

Baseline comparison:

- Raw-grayscale average CER: 33.31%
- Raw-grayscale average WER: 58.23%
- CER delta: +8.23 percentage points
- WER delta: +13.72 percentage points

Decision:

- Otsu binarization worsens OCR accuracy on the SROIE fixture subset.
- Keep binarization as an experiment only; do not add it to the default OCR preprocessing pipeline.

