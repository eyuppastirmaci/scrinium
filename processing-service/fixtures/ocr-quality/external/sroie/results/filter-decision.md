# SROIE OCR Filter Decision

The default OCR preprocessing pipeline is kept measurement-driven. On the SROIE fixture subset, only filters with a measurable OCR accuracy improvement remain in the default path.

| Filter / Variant | Average CER | Average WER | Baseline CER Delta | Baseline WER Delta | Default Decision |
| --- | ---: | ---: | ---: | ---: | --- |
| `raw-grayscale` | 33.31% | 58.23% | 0.00 pp | 0.00 pp | Keep as base conversion |
| `grayscale-deskew` | 32.94% | 57.21% | -0.37 pp | -1.02 pp | Keep |
| `grayscale-denoise` | 61.51% | 78.54% | +28.20 pp | +20.31 pp | Remove |
| `grayscale-dpi` | 33.31% | 58.23% | 0.00 pp | 0.00 pp | Remove from default for now |
| `grayscale-binarize` | 41.54% | 71.95% | +8.23 pp | +13.72 pp | Keep as experiment only |

Default pipeline after this decision:

```text
GrayscaleStep -> DeskewStep
```

Notes:

- `DenoiseStep` remains in code for future experiments, but it is removed from the default OCR pipeline because this median-filter configuration significantly worsened SROIE OCR accuracy.
- `DpiNormalizationStep` remains in code for future low-resolution fixture experiments, but it is removed from the default OCR pipeline because it had no measurable SROIE effect.
- Binarization was never added to the default pipeline and remains an explicit experiment only.
- `ContrastNormalizationStep` remains available in code, but is not kept in the default pipeline because it was not part of the measured SROIE-winning path.

