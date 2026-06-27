use crate::domain::model::{GeneratedThumbnail, ThumbnailSize};

#[derive(Debug)]
pub struct ThumbnailError(pub String);

pub trait ThumbnailGenerator: Send + Sync {
    fn generate(
        &self,
        content: &[u8],
        content_type: &str,
        size: ThumbnailSize,
    ) -> Result<GeneratedThumbnail, ThumbnailError>;
}
