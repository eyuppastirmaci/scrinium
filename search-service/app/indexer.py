"""Search index persistence operations for upserting and deleting documents."""

import uuid

import asyncpg


async def upsert_document(
    pool: asyncpg.Pool,
    document_id: uuid.UUID,
    file_name: str,
    content: str,
    metadata_text: str,
) -> None:
    """Insert or update a document in the search index.

    Uses ON CONFLICT to make the operation idempotent — safe for duplicate
    Kafka deliveries. The GIN full-text index is updated automatically by
    PostgreSQL when the row changes.

    Args:
        pool: asyncpg connection pool.
        document_id: Unique identifier of the document.
        file_name: Original file name (weighted highest in search ranking).
        content: Combined extracted text from all pages.
        metadata_text: Searchable metadata fields joined into a single string.
    """
    await pool.execute(
        """
        INSERT INTO search_index (document_id, file_name, content, metadata_text)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (document_id) DO UPDATE
        SET file_name = EXCLUDED.file_name,
            content = EXCLUDED.content,
            metadata_text = EXCLUDED.metadata_text,
            updated_at = now()
        """,
        document_id,
        file_name,
        content,
        metadata_text,
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
