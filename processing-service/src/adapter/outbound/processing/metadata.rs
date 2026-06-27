use crate::domain::model::ExtractedDocumentMetadata;
use crate::domain::port::{MetadataExtractionError, MetadataExtractionInput, MetadataExtractor};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use exif::{Reader, Tag, Value};
use lopdf::{Dictionary, Document, Object};
use serde_json::json;
use std::io::Cursor;
use whatlang::detect;

const MIN_LANGUAGE_DETECTION_CHARS: usize = 20;
const MIN_LANGUAGE_DETECTION_CONFIDENCE: f64 = 0.35;

pub struct NoopMetadataExtractor;

#[async_trait::async_trait]
impl MetadataExtractor for NoopMetadataExtractor {
    async fn extract(
        &self,
        _input: MetadataExtractionInput<'_>,
    ) -> Result<ExtractedDocumentMetadata, MetadataExtractionError> {
        Ok(ExtractedDocumentMetadata::default())
    }
}

pub struct PdfMetadataExtractor;

impl PdfMetadataExtractor {
    pub fn new() -> Self {
        Self
    }

    fn extract_pdf(
        &self,
        content: &[u8],
    ) -> Result<ExtractedDocumentMetadata, MetadataExtractionError> {
        let doc = Document::load_mem(content)
            .map_err(|e| MetadataExtractionError(format!("PDF metadata extraction failed: {e}")))?;

        let mut metadata = ExtractedDocumentMetadata {
            page_count: Some(doc.get_pages().len() as i32),
            ..ExtractedDocumentMetadata::default()
        };

        if let Some(info) = pdf_info_dictionary(&doc) {
            metadata.pdf_created_at =
                pdf_info_string(info, b"CreationDate").and_then(|date| parse_pdf_date(&date));
            metadata.pdf_modified_at =
                pdf_info_string(info, b"ModDate").and_then(|date| parse_pdf_date(&date));
            metadata.pdf_author = pdf_info_string(info, b"Author").and_then(non_empty_string);
            metadata.pdf_title = pdf_info_string(info, b"Title").and_then(non_empty_string);
        }

        metadata.metadata_json = json!({
            "pdf": {
                "hasDocumentInfo": pdf_info_dictionary(&doc).is_some()
            }
        });

        Ok(metadata)
    }
}

#[async_trait::async_trait]
impl MetadataExtractor for PdfMetadataExtractor {
    async fn extract(
        &self,
        input: MetadataExtractionInput<'_>,
    ) -> Result<ExtractedDocumentMetadata, MetadataExtractionError> {
        if input.content_type != "application/pdf" {
            return Ok(ExtractedDocumentMetadata::default());
        }

        self.extract_pdf(input.content)
    }
}

pub struct ImageExifMetadataExtractor;

impl ImageExifMetadataExtractor {
    pub fn new() -> Self {
        Self
    }

    fn extract_image(
        &self,
        content: &[u8],
    ) -> Result<ExtractedDocumentMetadata, MetadataExtractionError> {
        let exif = match Reader::new().read_from_container(&mut Cursor::new(content)) {
            Ok(exif) => exif,
            Err(_) => return Ok(ExtractedDocumentMetadata::default()),
        };

        let image_captured_at = find_exif_field(&exif, Tag::DateTimeOriginal)
            .or_else(|| find_exif_field(&exif, Tag::DateTimeDigitized))
            .or_else(|| find_exif_field(&exif, Tag::DateTime))
            .and_then(|field| exif_ascii(&field.value))
            .and_then(|date| parse_exif_datetime(&date));

        let make = find_exif_field(&exif, Tag::Make)
            .and_then(|field| exif_ascii(&field.value))
            .and_then(non_empty_string);
        let model = find_exif_field(&exif, Tag::Model)
            .and_then(|field| exif_ascii(&field.value))
            .and_then(non_empty_string);

        let image_device = combine_device(make.as_deref(), model.as_deref());
        let image_gps_present = find_exif_field(&exif, Tag::GPSInfoIFDPointer).is_some()
            || exif.fields().any(|field| is_gps_tag(field.tag));

        Ok(ExtractedDocumentMetadata {
            image_captured_at,
            image_device,
            image_gps_present,
            image_gps_redacted: image_gps_present,
            metadata_json: json!({
                "image": {
                    "exifPresent": true,
                    "gpsRedactionPolicy": if image_gps_present { "coordinates_redacted" } else { "none" }
                }
            }),
            ..ExtractedDocumentMetadata::default()
        })
    }
}

#[async_trait::async_trait]
impl MetadataExtractor for ImageExifMetadataExtractor {
    async fn extract(
        &self,
        input: MetadataExtractionInput<'_>,
    ) -> Result<ExtractedDocumentMetadata, MetadataExtractionError> {
        if !input.content_type.starts_with("image/") {
            return Ok(ExtractedDocumentMetadata::default());
        }

        self.extract_image(input.content)
    }
}

pub struct LanguageMetadataExtractor;

impl LanguageMetadataExtractor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl MetadataExtractor for LanguageMetadataExtractor {
    async fn extract(
        &self,
        input: MetadataExtractionInput<'_>,
    ) -> Result<ExtractedDocumentMetadata, MetadataExtractionError> {
        let Some(text) = input.extracted_text else {
            return Ok(ExtractedDocumentMetadata::default());
        };

        let normalized = normalize_language_detection_text(text);
        if normalized.chars().count() < MIN_LANGUAGE_DETECTION_CHARS {
            return Ok(ExtractedDocumentMetadata::default());
        }

        let Some(info) = detect(&normalized) else {
            return Ok(ExtractedDocumentMetadata::default());
        };

        if info.confidence() < MIN_LANGUAGE_DETECTION_CONFIDENCE {
            return Ok(ExtractedDocumentMetadata::default());
        }

        Ok(ExtractedDocumentMetadata {
            detected_language: Some(info.lang().code().to_string()),
            metadata_json: json!({
                "language": {
                    "confidence": info.confidence()
                }
            }),
            ..ExtractedDocumentMetadata::default()
        })
    }
}

pub struct CompositeMetadataExtractor {
    extractors: Vec<Box<dyn MetadataExtractor>>,
}

impl CompositeMetadataExtractor {
    pub fn new(extractors: Vec<Box<dyn MetadataExtractor>>) -> Self {
        Self { extractors }
    }
}

#[async_trait::async_trait]
impl MetadataExtractor for CompositeMetadataExtractor {
    async fn extract(
        &self,
        input: MetadataExtractionInput<'_>,
    ) -> Result<ExtractedDocumentMetadata, MetadataExtractionError> {
        let mut combined = ExtractedDocumentMetadata::default();

        for extractor in &self.extractors {
            let extracted = extractor
                .extract(MetadataExtractionInput {
                    content: input.content,
                    content_type: input.content_type,
                    extracted_text: input.extracted_text,
                })
                .await?;
            merge_metadata(&mut combined, extracted);
        }

        Ok(combined)
    }
}

fn pdf_info_dictionary(doc: &Document) -> Option<&Dictionary> {
    let info = doc.trailer.get(b"Info").ok()?;
    let info = match info {
        Object::Reference(id) => doc.get_object(*id).ok()?,
        object => object,
    };

    info.as_dict().ok()
}

fn pdf_info_string(info: &Dictionary, key: &[u8]) -> Option<String> {
    let object = info.get(key).ok()?;

    match object {
        Object::String(bytes, _) => decode_pdf_string(bytes),
        Object::Name(bytes) => String::from_utf8(bytes.clone()).ok(),
        _ => None,
    }
}

fn decode_pdf_string(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        return String::from_utf16(&utf16).ok();
    }

    Some(String::from_utf8_lossy(bytes).into_owned())
}

fn non_empty_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn find_exif_field(exif: &exif::Exif, tag: Tag) -> Option<&exif::Field> {
    exif.fields().find(|field| field.tag == tag)
}

fn is_gps_tag(tag: Tag) -> bool {
    matches!(
        tag,
        Tag::GPSVersionID
            | Tag::GPSLatitudeRef
            | Tag::GPSLatitude
            | Tag::GPSLongitudeRef
            | Tag::GPSLongitude
            | Tag::GPSAltitudeRef
            | Tag::GPSAltitude
            | Tag::GPSTimeStamp
            | Tag::GPSDateStamp
    )
}

fn exif_ascii(value: &Value) -> Option<String> {
    match value {
        Value::Ascii(values) => values.iter().find(|value| !value.is_empty()).map(|value| {
            String::from_utf8_lossy(value)
                .trim_end_matches('\0')
                .to_string()
        }),
        _ => None,
    }
}

fn combine_device(make: Option<&str>, model: Option<&str>) -> Option<String> {
    match (make, model) {
        (Some(make), Some(model)) if model.starts_with(make) => Some(model.to_string()),
        (Some(make), Some(model)) => Some(format!("{make} {model}")),
        (Some(make), None) => Some(make.to_string()),
        (None, Some(model)) => Some(model.to_string()),
        (None, None) => None,
    }
}

fn parse_exif_datetime(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim().trim_end_matches('\0');
    let naive = NaiveDateTime::parse_from_str(value, "%Y:%m:%d %H:%M:%S").ok()?;
    Some(Utc.from_utc_datetime(&naive))
}

fn normalize_language_detection_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn merge_metadata(target: &mut ExtractedDocumentMetadata, source: ExtractedDocumentMetadata) {
    target.page_count = target.page_count.or(source.page_count);
    target.pdf_created_at = target.pdf_created_at.or(source.pdf_created_at);
    target.pdf_modified_at = target.pdf_modified_at.or(source.pdf_modified_at);
    target.pdf_author = target.pdf_author.take().or(source.pdf_author);
    target.pdf_title = target.pdf_title.take().or(source.pdf_title);
    target.image_captured_at = target.image_captured_at.or(source.image_captured_at);
    target.image_device = target.image_device.take().or(source.image_device);
    target.image_gps_present |= source.image_gps_present;
    target.image_gps_redacted |= source.image_gps_redacted;
    target.detected_language = target.detected_language.take().or(source.detected_language);
    merge_json_objects(&mut target.metadata_json, source.metadata_json);
}

fn merge_json_objects(target: &mut serde_json::Value, source: serde_json::Value) {
    let (Some(target), Some(source)) = (target.as_object_mut(), source.as_object()) else {
        return;
    };

    for (key, value) in source {
        target.insert(key.clone(), value.clone());
    }
}

fn parse_pdf_date(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim().strip_prefix("D:").unwrap_or(value.trim());
    if value.len() < 4 {
        return None;
    }

    let year = parse_date_part(value, 0, 4)?;
    let month = parse_date_part(value, 4, 2).unwrap_or(1);
    let day = parse_date_part(value, 6, 2).unwrap_or(1);
    let hour = parse_date_part(value, 8, 2).unwrap_or(0);
    let minute = parse_date_part(value, 10, 2).unwrap_or(0);
    let second = parse_date_part(value, 12, 2).unwrap_or(0);

    let date = NaiveDate::from_ymd_opt(year.try_into().ok()?, month, day)?;
    let time = NaiveTime::from_hms_opt(hour, minute, second)?;
    let naive = date.and_time(time);
    let tz_start = 14;

    if value.len() <= tz_start {
        return Some(Utc.from_utc_datetime(&naive));
    }

    let timezone = &value[tz_start..];
    if timezone.starts_with('Z') {
        return Some(Utc.from_utc_datetime(&naive));
    }

    let sign = timezone.chars().next()?;
    if sign != '+' && sign != '-' {
        return Some(Utc.from_utc_datetime(&naive));
    }

    let offset_digits: String = timezone[1..]
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    let offset_hour = parse_date_part(&offset_digits, 0, 2).unwrap_or(0);
    let offset_minute = parse_date_part(&offset_digits, 2, 2).unwrap_or(0);
    let offset_seconds = (offset_hour * 3600 + offset_minute * 60) as i32;
    let offset = if sign == '-' {
        FixedOffset::west_opt(offset_seconds)?
    } else {
        FixedOffset::east_opt(offset_seconds)?
    };

    offset
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
}

fn parse_date_part(value: &str, start: usize, len: usize) -> Option<u32> {
    value.get(start..start + len)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{Object, Stream, dictionary};

    #[tokio::test]
    async fn noop_metadata_extractor_returns_empty_metadata() {
        let extractor = NoopMetadataExtractor;

        let metadata = extractor
            .extract(MetadataExtractionInput {
                content: b"not inspected yet",
                content_type: "application/pdf",
                extracted_text: Some("Merhaba dunya"),
            })
            .await
            .expect("noop extraction should succeed");

        assert!(metadata.is_empty());
    }

    #[tokio::test]
    async fn pdf_metadata_extractor_reads_page_count_and_document_info() {
        let pdf = build_test_pdf(
            2,
            Some("D:20240102112233+03'00'"),
            Some("D:20240203122334Z"),
            Some("Ayse Yilmaz"),
            Some("Invoice 42"),
        );
        let extractor = PdfMetadataExtractor::new();

        let metadata = extractor
            .extract(MetadataExtractionInput {
                content: &pdf,
                content_type: "application/pdf",
                extracted_text: None,
            })
            .await
            .expect("PDF metadata extraction should succeed");

        assert_eq!(metadata.page_count, Some(2));
        assert_eq!(metadata.pdf_author.as_deref(), Some("Ayse Yilmaz"));
        assert_eq!(metadata.pdf_title.as_deref(), Some("Invoice 42"));
        assert_eq!(
            metadata.pdf_created_at.map(|dt| dt.to_rfc3339()),
            Some("2024-01-02T08:22:33+00:00".to_string())
        );
        assert_eq!(
            metadata.pdf_modified_at.map(|dt| dt.to_rfc3339()),
            Some("2024-02-03T12:23:34+00:00".to_string())
        );
    }

    #[tokio::test]
    async fn pdf_metadata_extractor_ignores_missing_document_info_fields() {
        let pdf = build_test_pdf(1, None, None, None, None);
        let extractor = PdfMetadataExtractor::new();

        let metadata = extractor
            .extract(MetadataExtractionInput {
                content: &pdf,
                content_type: "application/pdf",
                extracted_text: None,
            })
            .await
            .expect("PDF metadata extraction should succeed");

        assert_eq!(metadata.page_count, Some(1));
        assert_eq!(metadata.pdf_created_at, None);
        assert_eq!(metadata.pdf_modified_at, None);
        assert_eq!(metadata.pdf_author, None);
        assert_eq!(metadata.pdf_title, None);
    }

    #[tokio::test]
    async fn pdf_metadata_extractor_returns_empty_for_non_pdf_content() {
        let extractor = PdfMetadataExtractor::new();

        let metadata = extractor
            .extract(MetadataExtractionInput {
                content: b"image bytes",
                content_type: "image/jpeg",
                extracted_text: None,
            })
            .await
            .expect("non-PDF extraction should succeed");

        assert!(metadata.is_empty());
    }

    #[tokio::test]
    async fn image_exif_metadata_extractor_reads_capture_date_device_and_redacts_gps() {
        let image = build_test_tiff_with_exif(true);
        let extractor = ImageExifMetadataExtractor::new();

        let metadata = extractor
            .extract(MetadataExtractionInput {
                content: &image,
                content_type: "image/tiff",
                extracted_text: None,
            })
            .await
            .expect("image EXIF extraction should succeed");

        assert_eq!(
            metadata.image_captured_at.map(|dt| dt.to_rfc3339()),
            Some("2024-05-06T07:08:09+00:00".to_string())
        );
        assert_eq!(metadata.image_device.as_deref(), Some("Canon EOS R50"));
        assert!(metadata.image_gps_present);
        assert!(metadata.image_gps_redacted);
        assert!(!metadata.metadata_json.to_string().contains("latitude"));
        assert!(!metadata.metadata_json.to_string().contains("longitude"));
    }

    #[tokio::test]
    async fn image_exif_metadata_extractor_leaves_gps_flags_false_when_absent() {
        let image = build_test_tiff_with_exif(false);
        let extractor = ImageExifMetadataExtractor::new();

        let metadata = extractor
            .extract(MetadataExtractionInput {
                content: &image,
                content_type: "image/tiff",
                extracted_text: None,
            })
            .await
            .expect("image EXIF extraction should succeed");

        assert_eq!(metadata.image_device.as_deref(), Some("Canon EOS R50"));
        assert!(!metadata.image_gps_present);
        assert!(!metadata.image_gps_redacted);
    }

    #[tokio::test]
    async fn image_exif_metadata_extractor_returns_empty_for_images_without_exif() {
        let extractor = ImageExifMetadataExtractor::new();

        let metadata = extractor
            .extract(MetadataExtractionInput {
                content: b"not a readable image container",
                content_type: "image/jpeg",
                extracted_text: None,
            })
            .await
            .expect("missing EXIF should not fail extraction");

        assert!(metadata.is_empty());
    }

    #[tokio::test]
    async fn language_metadata_extractor_detects_turkish_text() {
        let extractor = LanguageMetadataExtractor::new();

        let metadata = extractor
            .extract(MetadataExtractionInput {
                content: b"",
                content_type: "application/pdf",
                extracted_text: Some(
                    "Bu fatura toplam tutar ve odeme bilgilerini iceren Turkce bir belgedir.",
                ),
            })
            .await
            .expect("language detection should succeed");

        assert_eq!(metadata.detected_language.as_deref(), Some("tur"));
    }

    #[tokio::test]
    async fn language_metadata_extractor_detects_english_text() {
        let extractor = LanguageMetadataExtractor::new();

        let metadata = extractor
            .extract(MetadataExtractionInput {
                content: b"",
                content_type: "application/pdf",
                extracted_text: Some(
                    "This invoice contains payment details, customer information, and totals.",
                ),
            })
            .await
            .expect("language detection should succeed");

        assert_eq!(metadata.detected_language.as_deref(), Some("eng"));
    }

    #[tokio::test]
    async fn language_metadata_extractor_ignores_short_or_missing_text() {
        let extractor = LanguageMetadataExtractor::new();

        let short = extractor
            .extract(MetadataExtractionInput {
                content: b"",
                content_type: "application/pdf",
                extracted_text: Some("Kisa metin"),
            })
            .await
            .expect("short text should not fail");
        assert!(short.is_empty());

        let missing = extractor
            .extract(MetadataExtractionInput {
                content: b"",
                content_type: "application/pdf",
                extracted_text: None,
            })
            .await
            .expect("missing text should not fail");
        assert!(missing.is_empty());
    }

    #[test]
    fn parse_exif_datetime_reads_camera_timestamp_as_utc() {
        assert_eq!(
            parse_exif_datetime("2024:05:06 07:08:09").map(|dt| dt.to_rfc3339()),
            Some("2024-05-06T07:08:09+00:00".to_string())
        );
    }

    #[test]
    fn normalize_language_detection_text_collapses_whitespace() {
        assert_eq!(
            normalize_language_detection_text("  bu\n\nbir\tmetin  "),
            "bu bir metin"
        );
    }

    #[test]
    fn parse_pdf_date_supports_partial_and_offset_dates() {
        assert_eq!(
            parse_pdf_date("D:202401").map(|dt| dt.to_rfc3339()),
            Some("2024-01-01T00:00:00+00:00".to_string())
        );
        assert_eq!(
            parse_pdf_date("D:20240102112233-05'30'").map(|dt| dt.to_rfc3339()),
            Some("2024-01-02T16:52:33+00:00".to_string())
        );
    }

    fn build_test_pdf(
        page_count: usize,
        created_at: Option<&str>,
        modified_at: Option<&str>,
        author: Option<&str>,
        title: Option<&str>,
    ) -> Vec<u8> {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let catalog_id = doc.new_object_id();
        let info_id = doc.new_object_id();

        let mut kids = Vec::with_capacity(page_count);
        for _ in 0..page_count {
            let page_id = doc.new_object_id();
            let content_id = doc.add_object(Stream::new(dictionary! {}, Vec::new()));
            let page = dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
                "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
                "Contents" => content_id,
                "Resources" => dictionary! {}
            };
            doc.objects.insert(page_id, Object::Dictionary(page));
            kids.push(Object::Reference(page_id));
        }

        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => Object::Array(kids),
                "Count" => page_count as i64
            }),
        );
        doc.objects.insert(
            catalog_id,
            Object::Dictionary(dictionary! {
                "Type" => "Catalog",
                "Pages" => pages_id
            }),
        );

        let mut info = Dictionary::new();
        if let Some(value) = created_at {
            info.set("CreationDate", Object::string_literal(value));
        }
        if let Some(value) = modified_at {
            info.set("ModDate", Object::string_literal(value));
        }
        if let Some(value) = author {
            info.set("Author", Object::string_literal(value));
        }
        if let Some(value) = title {
            info.set("Title", Object::string_literal(value));
        }
        doc.objects.insert(info_id, Object::Dictionary(info));

        doc.trailer.set("Root", catalog_id);
        doc.trailer.set("Info", info_id);

        let mut buffer = Vec::new();
        doc.save_to(&mut buffer)
            .expect("test PDF should be writable");
        buffer
    }

    fn build_test_tiff_with_exif(include_gps: bool) -> Vec<u8> {
        let entry_count = if include_gps { 4_u16 } else { 3_u16 };
        let ifd_start = 8_usize;
        let ifd_size = 2 + entry_count as usize * 12 + 4;
        let data_start = ifd_start + ifd_size;
        let exif_ifd_offset = data_start + ascii_len("Canon") + ascii_len("EOS R50");
        let exif_ifd_size = 2 + 12 + 4;
        let capture_date_offset = exif_ifd_offset + exif_ifd_size;
        let gps_ifd_offset = capture_date_offset + ascii_len("2024:05:06 07:08:09");

        let mut data = Vec::new();
        data.extend_from_slice(b"MM");
        data.extend_from_slice(&42_u16.to_be_bytes());
        data.extend_from_slice(&(ifd_start as u32).to_be_bytes());
        data.extend_from_slice(&entry_count.to_be_bytes());

        let mut values = Vec::new();
        write_ascii_entry(&mut data, &mut values, 0x010F, "Canon", data_start);
        write_ascii_entry(&mut data, &mut values, 0x0110, "EOS R50", data_start);
        write_long_entry(&mut data, 0x8769, exif_ifd_offset as u32);

        if include_gps {
            write_long_entry(&mut data, 0x8825, gps_ifd_offset as u32);
        }

        data.extend_from_slice(&0_u32.to_be_bytes());
        data.extend_from_slice(&values);
        data.extend_from_slice(&1_u16.to_be_bytes());
        write_ascii_entry_at_offset(
            &mut data,
            0x9003,
            "2024:05:06 07:08:09",
            capture_date_offset,
        );
        data.extend_from_slice(&0_u32.to_be_bytes());
        data.extend_from_slice(b"2024:05:06 07:08:09\0");

        if include_gps {
            data.extend_from_slice(&1_u16.to_be_bytes());
            write_inline_ascii_entry(&mut data, 0x0001, "N");
            data.extend_from_slice(&0_u32.to_be_bytes());
        }

        data
    }

    fn ascii_len(value: &str) -> usize {
        value.len() + 1
    }

    fn write_ascii_entry(
        entries: &mut Vec<u8>,
        values: &mut Vec<u8>,
        tag: u16,
        value: &str,
        data_start: usize,
    ) {
        let offset = data_start + values.len();
        let count = ascii_len(value) as u32;

        entries.extend_from_slice(&tag.to_be_bytes());
        entries.extend_from_slice(&2_u16.to_be_bytes());
        entries.extend_from_slice(&count.to_be_bytes());
        entries.extend_from_slice(&(offset as u32).to_be_bytes());

        values.extend_from_slice(value.as_bytes());
        values.push(0);
    }

    fn write_long_entry(entries: &mut Vec<u8>, tag: u16, value: u32) {
        entries.extend_from_slice(&tag.to_be_bytes());
        entries.extend_from_slice(&4_u16.to_be_bytes());
        entries.extend_from_slice(&1_u32.to_be_bytes());
        entries.extend_from_slice(&value.to_be_bytes());
    }

    fn write_ascii_entry_at_offset(entries: &mut Vec<u8>, tag: u16, value: &str, offset: usize) {
        entries.extend_from_slice(&tag.to_be_bytes());
        entries.extend_from_slice(&2_u16.to_be_bytes());
        entries.extend_from_slice(&(ascii_len(value) as u32).to_be_bytes());
        entries.extend_from_slice(&(offset as u32).to_be_bytes());
    }

    fn write_inline_ascii_entry(entries: &mut Vec<u8>, tag: u16, value: &str) {
        let mut inline_value = [0_u8; 4];
        for (index, byte) in value.as_bytes().iter().take(3).enumerate() {
            inline_value[index] = *byte;
        }

        entries.extend_from_slice(&tag.to_be_bytes());
        entries.extend_from_slice(&2_u16.to_be_bytes());
        entries.extend_from_slice(&(ascii_len(value) as u32).to_be_bytes());
        entries.extend_from_slice(&inline_value);
    }
}
