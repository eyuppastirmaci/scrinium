use crate::domain::model::ExtractedDocumentMetadata;

pub struct MetadataExtractionInput<'a> {
    pub content: &'a [u8],
    pub content_type: &'a str,
    pub extracted_text: Option<&'a str>,
}

#[derive(Debug)]
pub struct MetadataExtractionError(pub String);

#[async_trait::async_trait]
pub trait MetadataExtractor: Send + Sync {
    async fn extract(
        &self,
        input: MetadataExtractionInput<'_>,
    ) -> Result<ExtractedDocumentMetadata, MetadataExtractionError>;
}
