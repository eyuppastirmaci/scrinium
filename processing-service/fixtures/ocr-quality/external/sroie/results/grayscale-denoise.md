# SROIE Grayscale + Denoise OCR Measurement

Command:

```powershell
cargo run --bin ocr_quality -- --fixtures fixtures/ocr-quality/external/sroie/manifest.toml --variant grayscale-denoise
```

Configuration:

- Variant: `grayscale-denoise`
- Language: `eng`
- Tesseract: local Tesseract executable from `PROCESSING_TESSERACT_PATH`
- Fixtures: 10

| Fixture | CER | WER | Truth Chars | OCR Chars |
| --- | ---: | ---: | ---: | ---: |
| `sroie_000` | 65.98% | 82.35% | 485 | 260 |
| `sroie_001` | 99.71% | 100.00% | 684 | 13 |
| `sroie_002` | 15.35% | 42.28% | 723 | 692 |
| `sroie_003` | 63.18% | 85.71% | 584 | 342 |
| `sroie_004` | 99.12% | 100.00% | 797 | 22 |
| `sroie_005` | 47.59% | 74.19% | 374 | 373 |
| `sroie_006` | 33.08% | 60.51% | 916 | 865 |
| `sroie_007` | 42.86% | 67.12% | 441 | 359 |
| `sroie_008` | 90.35% | 95.54% | 891 | 113 |
| `sroie_009` | 57.91% | 77.69% | 784 | 781 |

Summary:

- Average CER: 61.51%
- Average WER: 78.54%

Baseline comparison:

- Raw-grayscale average CER: 33.31%
- Raw-grayscale average WER: 58.23%
- CER delta: +28.20 percentage points
- WER delta: +20.31 percentage points

Decision:

- The current median-filter denoise step significantly worsens OCR accuracy on the SROIE fixture subset.
- Do not rely on this denoise configuration as default behavior without changing and re-measuring it.

