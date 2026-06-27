# SROIE Grayscale + Deskew OCR Measurement

Command:

```powershell
cargo run --bin ocr_quality -- --fixtures fixtures/ocr-quality/external/sroie/manifest.toml --variant grayscale-deskew
```

Configuration:

- Variant: `grayscale-deskew`
- Language: `eng`
- Tesseract: local Tesseract executable from `PROCESSING_TESSERACT_PATH`
- Fixtures: 10

| Fixture | CER | WER | Truth Chars | OCR Chars |
| --- | ---: | ---: | ---: | ---: |
| `sroie_000` | 32.37% | 57.65% | 485 | 459 |
| `sroie_001` | 48.25% | 67.65% | 684 | 478 |
| `sroie_002` | 9.27% | 32.52% | 723 | 719 |
| `sroie_003` | 37.84% | 68.57% | 584 | 501 |
| `sroie_004` | 20.33% | 54.17% | 797 | 751 |
| `sroie_005` | 43.85% | 66.13% | 374 | 342 |
| `sroie_006` | 28.17% | 42.68% | 916 | 907 |
| `sroie_007` | 22.22% | 52.05% | 441 | 454 |
| `sroie_008` | 27.95% | 49.68% | 891 | 821 |
| `sroie_009` | 59.18% | 80.99% | 784 | 784 |

Summary:

- Average CER: 32.94%
- Average WER: 57.21%

Baseline comparison:

- Raw-grayscale average CER: 33.31%
- Raw-grayscale average WER: 58.23%
- CER delta: -0.37 percentage points
- WER delta: -1.02 percentage points

Decision:

- Deskew shows a small measurable improvement on the SROIE fixture subset.

