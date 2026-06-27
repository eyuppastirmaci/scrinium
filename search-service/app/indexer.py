"""Search index persistence operations for upserting and deleting documents."""

import uuid
from datetime import datetime, timezone

import asyncpg


async def upsert_document(
    pool: asyncpg.Pool,
    document_id: uuid.UUID,
    file_name: str,
    content_type: str,
    content: str,
    metadata_text: str,
    page_count: int | None,
    document_date: datetime | None,
    created_at: datetime | None,
) -> None:
    """Insert or update a document in the search index.

    Uses ON CONFLICT to make the operation idempotent — safe for duplicate
    event deliveries. The GIN full-text index is updated automatically by
    PostgreSQL when the row changes.

    Args:
        pool: asyncpg connection pool.
        document_id: Unique identifier of the document.
        file_name: Original file name (weighted highest in search ranking).
        content_type: MIME type of the document (e.g. "application/pdf").
        content: Combined extracted text from all pages.
        metadata_text: Searchable metadata fields joined into a single string.
        page_count: Number of pages in the document, or None if unknown.
        document_date: Document creation date (PDF created or image captured), or None.
        created_at: Upload timestamp, or None.
    """
    await pool.execute(
        """
        INSERT INTO search_index (document_id, file_name, content_type, content,
                                  metadata_text, page_count, document_date, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, now()))
        ON CONFLICT (document_id) DO UPDATE
        SET file_name = EXCLUDED.file_name,
            content_type = EXCLUDED.content_type,
            content = EXCLUDED.content,
            metadata_text = EXCLUDED.metadata_text,
            page_count = EXCLUDED.page_count,
            document_date = EXCLUDED.document_date,
            created_at = EXCLUDED.created_at,
            updated_at = now()
        """,
        document_id,
        file_name,
        content_type,
        content,
        metadata_text,
        page_count,
        document_date,
        created_at,
    )


async def delete_document(pool: asyncpg.Pool, document_id: uuid.UUID) -> None:
    """Remove a document from the search index.

    Called when a document.deleted event is received. No-op if the document
    is not in the index.

    Args:
        pool: asyncpg connection pool.
        document_id: Unique identifier of the document to remove.
    """
    await pool.execute(
        "DELETE FROM search_index WHERE document_id = $1",
        document_id,
    )


def build_metadata_text(metadata: dict) -> str:
    """Extract searchable text from document metadata fields.

    Joins non-null values of title, author, device, and language into a
    single space-separated string for full-text indexing.

    Args:
        metadata: Metadata dict from the processing.completed event payload.

    Returns:
        Space-separated string of metadata values, or empty string if none.
    """
    parts: list[str] = []
    for field in ("pdfTitle", "pdfAuthor", "imageDevice", "detectedLanguage"):
        value = metadata.get(field)
        if value:
            parts.append(value)
    return " ".join(parts)


def build_content_text(pages: list[dict]) -> str:
    """Combine extracted text from all pages into a single string.

    Pages are joined with double newlines to preserve page boundaries
    in the indexed content.

    Args:
        pages: List of page dicts, each with a "text" key.

    Returns:
        Combined text from all pages.
    """
    return "\n\n".join(p.get("text", "") for p in pages)


def extract_document_date(metadata: dict) -> datetime | None:
    """Extract the document's own date from metadata.

    Prefers PDF creation date, falls back to image capture date.

    Args:
        metadata: Metadata dict from the processing.completed event payload.

    Returns:
        Parsed datetime or None if no date is available.
    """
    for field in ("pdfCreatedAt", "imageCapturedAt"):
        value = metadata.get(field)
        if value:
            return _parse_iso(value)
    return None


def _parse_iso(value: str) -> datetime | None:
    """Parse an ISO-8601 datetime string, returning None on failure."""
    try:
        dt = datetime.fromisoformat(value)
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return dt
    except (ValueError, TypeError):
        return None
