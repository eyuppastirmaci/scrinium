use crate::domain::model::{GeneratedThumbnail, ThumbnailSize};
use crate::domain::port::{ThumbnailError, ThumbnailGenerator};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::DynamicImage;
use pdfium_render::prelude::*;
use std::io::Cursor;
use std::sync::Arc;

pub struct ImageThumbnailGenerator;

impl ImageThumbnailGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl ThumbnailGenerator for ImageThumbnailGenerator {
    fn generate(
        &self,
        content: &[u8],
        _content_type: &str,
        size: ThumbnailSize,
    ) -> Result<GeneratedThumbnail, ThumbnailError> {
        let img = image::load_from_memory(content)
            .map_err(|e| ThumbnailError(format!("failed to load image: {e}")))?;

        encode_thumbnail(img, size)
    }
}

pub struct PdfThumbnailGenerator {
    pdfium: Arc<Pdfium>,
}

impl PdfThumbnailGenerator {
    pub fn new(pdfium: Arc<Pdfium>) -> Self {
        Self { pdfium }
    }
}

impl ThumbnailGenerator for PdfThumbnailGenerator {
    fn generate(
        &self,
        content: &[u8],
        _content_type: &str,
        size: ThumbnailSize,
    ) -> Result<GeneratedThumbnail, ThumbnailError> {
        let doc = self
            .pdfium
            .load_pdf_from_byte_slice(content, None)
            .map_err(|e| ThumbnailError(format!("failed to load PDF: {e}")))?;

        let page = doc
            .pages()
            .first()
            .map_err(|e| ThumbnailError(format!("failed to get first page: {e}")))?;

        let render_config = PdfRenderConfig::new()
            .set_target_width(size.max_width() as i32 * 2)
            .set_maximum_height(size.max_width() as i32 * 3);

        let bitmap = page
            .render_with_config(&render_config)
            .map_err(|e| ThumbnailError(format!("render failed: {e}")))?;

        let dynamic_image = bitmap
            .as_image()
            .as_rgba8()
            .map(|rgba| DynamicImage::ImageRgba8(rgba.clone()))
            .ok_or_else(|| ThumbnailError("bitmap conversion failed".to_string()))?;

        encode_thumbnail(dynamic_image, size)
    }
}

pub struct CompositeThumbnailGenerator {
    pdf_generator: Option<Box<dyn ThumbnailGenerator>>,
    image_generator: Box<dyn ThumbnailGenerator>,
}

impl CompositeThumbnailGenerator {
    pub fn new(
        image_generator: Box<dyn ThumbnailGenerator>,
        pdf_generator: Option<Box<dyn ThumbnailGenerator>>,
    ) -> Self {
        Self {
            pdf_generator,
            image_generator,
        }
    }
}

impl ThumbnailGenerator for CompositeThumbnailGenerator {
    fn generate(
        &self,
        content: &[u8],
        content_type: &str,
        size: ThumbnailSize,
    ) -> Result<GeneratedThumbnail, ThumbnailError> {
        if content_type == "application/pdf" {
            match &self.pdf_generator {
                Some(g) => g.generate(content, content_type, size),
                None => Err(ThumbnailError(
                    "PDF thumbnail generation not available".to_string(),
                )),
            }
        } else if content_type.starts_with("image/") {
            self.image_generator.generate(content, content_type, size)
        } else {
            Err(ThumbnailError(format!(
                "unsupported content type for thumbnail: {content_type}"
            )))
        }
    }
}

fn encode_thumbnail(
    img: DynamicImage,
    size: ThumbnailSize,
) -> Result<GeneratedThumbnail, ThumbnailError> {
    let max_width = size.max_width();
    let resized = if img.width() > max_width {
        img.resize(max_width, u32::MAX, FilterType::Triangle)
    } else {
        img
    };

    let width = resized.width();
    let height = resized.height();
    let rgb = resized.to_rgb8();

    let mut buf = Cursor::new(Vec::new());
    let encoder = JpegEncoder::new_with_quality(&mut buf, size.jpeg_quality());
    rgb.write_with_encoder(encoder)
        .map_err(|e| ThumbnailError(format!("JPEG encoding failed: {e}")))?;

    Ok(GeneratedThumbnail {
        size,
        bytes: buf.into_inner(),
        width,
        height,
    })
}
