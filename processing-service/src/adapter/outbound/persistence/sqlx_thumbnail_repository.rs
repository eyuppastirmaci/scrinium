use crate::domain::model::ThumbnailSize;
use crate::domain::port::{ThumbnailRecord, ThumbnailRepository, ThumbnailStoreError};
use sqlx::PgPool;
use uuid::Uuid;

pub struct SqlxThumbnailRepository {
    pool: PgPool,
}

impl SqlxThumbnailRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ThumbnailRepository for SqlxThumbnailRepository {
    async fn upsert(
        &self,
        document_id: Uuid,
        size: ThumbnailSize,
        storage_key: &str,
        width: i32,
        height: i32,
    ) -> Result<(), ThumbnailStoreError> {
        sqlx::query(
            "INSERT INTO document_thumbnails (document_id, size, storage_key, width, height)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (document_id, size) DO UPDATE
             SET storage_key = EXCLUDED.storage_key,
                 width = EXCLUDED.width,
                 height = EXCLUDED.height",
        )
        .bind(document_id)
        .bind(size.suffix().to_uppercase())
        .bind(storage_key)
        .bind(width)
        .bind(height)
        .execute(&self.pool)
        .await
        .map_err(|e| ThumbnailStoreError(format!("thumbnail upsert failed: {e}")))?;

        Ok(())
    }

    async fn find_by_document_id_and_size(
        &self,
        document_id: Uuid,
        size: ThumbnailSize,
    ) -> Result<Option<ThumbnailRecord>, ThumbnailStoreError> {
        let row = sqlx::query_as::<_, ThumbnailRow>(
            "SELECT document_id, size, storage_key, width, height
             FROM document_thumbnails
             WHERE document_id = $1 AND size = $2",
        )
        .bind(document_id)
        .bind(size.suffix().to_uppercase())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ThumbnailStoreError(format!("thumbnail query failed: {e}")))?;

        Ok(row.map(|r| r.into_record()))
    }
}

#[derive(sqlx::FromRow)]
struct ThumbnailRow {
    document_id: Uuid,
    size: String,
    storage_key: String,
    width: i32,
    height: i32,
}

impl ThumbnailRow {
    fn into_record(self) -> ThumbnailRecord {
        let size = match self.size.as_str() {
            "SMALL" => ThumbnailSize::Small,
            "MEDIUM" => ThumbnailSize::Medium,
            _ => ThumbnailSize::Small,
        };
        ThumbnailRecord {
            document_id: self.document_id,
            size,
            storage_key: self.storage_key,
            width: self.width,
            height: self.height,
        }
    }
}
