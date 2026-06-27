#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbnailSize {
    Small,
    Medium,
}

impl ThumbnailSize {
    pub fn max_width(&self) -> u32 {
        match self {
            Self::Small => 280,
            Self::Medium => 560,
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
        }
    }

    pub fn jpeg_quality(&self) -> u8 {
        80
    }

    pub fn all() -> &'static [ThumbnailSize] {
        &[Self::Small, Self::Medium]
    }

    pub fn storage_key(document_id: uuid::Uuid, size: ThumbnailSize) -> String {
        format!("thumbnails/{}_{}.jpg", document_id, size.suffix())
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedThumbnail {
    pub size: ThumbnailSize,
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
