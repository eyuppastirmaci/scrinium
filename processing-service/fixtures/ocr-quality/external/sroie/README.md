# SROIE OCR Quality Fixtures

This directory contains a small subset of ICDAR 2019 SROIE receipt samples used to validate the OCR quality harness format.

Source dataset:
- GitHub mirror: https://github.com/zzzDavid/ICDAR-2019-SROIE
- Official challenge: https://rrc.cvc.uab.es/?ch=13

License / attribution:
- The fixture images and derived transcript ground-truth files come from the ICDAR 2019 SROIE dataset.
- The original dataset is attributed to the ICDAR 2019 Robust Reading Challenge on Scanned Receipts OCR and Information Extraction.
- These files are included as a small fixture subset for OCR quality validation and should retain this source attribution when redistributed.

Acknowledgement:
- We acknowledge the ICDAR 2019 SROIE challenge organizers and dataset contributors for making the scanned receipt dataset available to the research and OCR community.

The imported files are intentionally limited to 10 samples. They are not the Turkish golden fixtures for Scrinium; they exist to make the measurement harness concrete before the Turkish receipt/invoice set is added.

Repository note:
- The SROIE fixture images and derived ground-truth transcript files are not committed to this repository.
- Normal `processing-service` startup does not require these fixture files.
- The files are only required when running the OCR quality harness with `cargo run --bin ocr_quality`.

Directory layout:

```text
images/
  000.jpg
ground-truth/
  000.txt
manifest.toml
```

Ground-truth files were generated from the matching `data/box/*.csv` files by dropping the first eight bounding-box coordinate columns and keeping the transcript column.
