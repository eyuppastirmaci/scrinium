"""Search service HTTP endpoints."""

import logging
from typing import Optional

from fastapi import APIRouter, Query
from pydantic import BaseModel, ConfigDict
from pydantic.alias_generators import to_camel

from app.db import get_pool

logger = logging.getLogger(__name__)

router = APIRouter()


class CamelModel(BaseModel):
    model_config = ConfigDict(alias_generator=to_camel, populate_by_name=True)


class SearchResultItem(CamelModel):
    document_id: str
    file_name: str
    snippet: str
    score: float


class SearchResponse(CamelModel):
    query: str
    items: list[SearchResultItem]
    total_count: int
    page: int
    size: int


@router.get("/health")
async def health():
    return {"status": "ok"}


@router.get("/search", response_model=SearchResponse, response_model_by_alias=True)
async def search(
    q: str = Query(..., min_length=1, description="Search query"),
    page: int = Query(0, ge=0, description="Page number (zero-based)"),
    size: int = Query(20, ge=1, le=100, description="Results per page"),
) -> SearchResponse:
    """Search documents by file name, extracted text, and metadata.

    Uses PostgreSQL full-text search with weighted ranking (file name > metadata > content)
    and trigram similarity for typo tolerance. Returns highlighted snippets around matches.

    Args:
        q: Search query string.
        page: Zero-based page number.
        size: Number of results per page (max 100).

    Returns:
        SearchResponse with ranked results, snippets, and pagination info.
    """
    pool = get_pool()
    offset = page * size

    # Count total matches.
    total_count = await pool.fetchval(
        """
        SELECT COUNT(*) FROM search_index
        WHERE to_tsvector('simple', coalesce(file_name, '') || ' ' || coalesce(metadata_text, '') || ' ' || coalesce(content, ''))
              @@ plainto_tsquery('simple', $1)
           OR file_name % $1
        """,
        q,
    )

    # Fetch ranked results with highlighted snippets.
    rows = await pool.fetch(
        """
        WITH search_vector AS (
            SELECT
                document_id,
                file_name,
                content,
                setweight(to_tsvector('simple', coalesce(file_name, '')), 'A') ||
                setweight(to_tsvector('simple', coalesce(metadata_text, '')), 'B') ||
                setweight(to_tsvector('simple', coalesce(content, '')), 'C') AS tsv,
                plainto_tsquery('simple', $1) AS query
            FROM search_index
        )
        SELECT
            document_id,
            file_name,
            ts_headline('simple', content, query,
                'StartSel=<mark>, StopSel=</mark>, MaxWords=30, MinWords=10, MaxFragments=1'
            ) AS snippet,
            GREATEST(
                ts_rank(tsv, query),
                similarity(file_name, $1) * 0.5
            ) AS score
        FROM search_vector
        WHERE tsv @@ query OR file_name % $1
        ORDER BY score DESC
        LIMIT $2 OFFSET $3
        """,
        q,
        size,
        offset,
    )

    items = [
        SearchResultItem(
            document_id=str(row["document_id"]),
            file_name=row["file_name"],
            snippet=row["snippet"],
            score=round(float(row["score"]), 4),
        )
        for row in rows
    ]

    logger.debug("search query=%r returned %d/%d results", q, len(items), total_count)

    return SearchResponse(
        query=q,
        items=items,
        total_count=total_count,
        page=page,
        size=size,
    )
