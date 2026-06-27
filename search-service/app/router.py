"""Search service HTTP endpoints."""

import logging
from datetime import date

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
    query: str | None
    items: list[SearchResultItem]
    total_count: int
    page: int
    size: int


@router.get("/health")
async def health():
    return {"status": "ok"}


@router.get("/search", response_model=SearchResponse, response_model_by_alias=True)
async def search(
    q: str | None = Query(None, min_length=1, description="Search query (optional when using filters)"),
    type: str | None = Query(None, description="Content type filter (e.g. 'application/pdf' or 'image')"),
    date_from: date | None = Query(None, alias="dateFrom", description="Upload date range start (YYYY-MM-DD)"),
    date_to: date | None = Query(None, alias="dateTo", description="Upload date range end (YYYY-MM-DD)"),
    doc_date_from: date | None = Query(None, alias="docDateFrom", description="Document date range start"),
    doc_date_to: date | None = Query(None, alias="docDateTo", description="Document date range end"),
    min_pages: int | None = Query(None, alias="minPages", ge=1, description="Minimum page count"),
    max_pages: int | None = Query(None, alias="maxPages", ge=1, description="Maximum page count"),
    page: int = Query(0, ge=0, description="Page number (zero-based)"),
    size: int = Query(20, ge=1, le=100, description="Results per page"),
) -> SearchResponse:
    """Search documents by text query and/or filters.

    Supports full-text search with weighted ranking, trigram similarity for typo tolerance,
    and filtering by content type, upload date, document date, and page count.
    At least one of query or filters must be provided.

    Args:
        q: Full-text search query (optional if filters are provided).
        type: Content type filter — exact match or prefix (e.g. "image" matches "image/png").
        date_from: Include documents uploaded on or after this date.
        date_to: Include documents uploaded on or before this date.
        doc_date_from: Include documents with document date on or after this date.
        doc_date_to: Include documents with document date on or before this date.
        min_pages: Include documents with at least this many pages.
        max_pages: Include documents with at most this many pages.
        page: Zero-based page number for pagination.
        size: Number of results per page (max 100).

    Returns:
        SearchResponse with ranked results, highlighted snippets, and pagination info.
    """
    has_filters = any([type, date_from, date_to, doc_date_from, doc_date_to, min_pages, max_pages])
    if not q and not has_filters:
        return SearchResponse(query=q, items=[], total_count=0, page=page, size=size)

    pool = get_pool()
    offset = page * size

    # Build dynamic WHERE conditions and parameter list.
    conditions: list[str] = []
    params: list = []
    param_idx = 1

    if q:
        conditions.append(f"""(
            (setweight(to_tsvector('simple', coalesce(file_name, '')), 'A') ||
             setweight(to_tsvector('simple', coalesce(metadata_text, '')), 'B') ||
             setweight(to_tsvector('simple', coalesce(content, '')), 'C'))
            @@ plainto_tsquery('simple', ${param_idx})
            OR file_name % ${param_idx}
        )""")
        params.append(q)
        param_idx += 1

    if type:
        if "/" in type:
            conditions.append(f"content_type = ${param_idx}")
        else:
            conditions.append(f"content_type LIKE ${param_idx}")
            type = type + "/%"
        params.append(type)
        param_idx += 1

    if date_from:
        conditions.append(f"created_at >= ${param_idx}")
        params.append(date_from)
        param_idx += 1

    if date_to:
        conditions.append(f"created_at < ${param_idx}::date + 1")
        params.append(date_to)
        param_idx += 1

    if doc_date_from:
        conditions.append(f"document_date >= ${param_idx}")
        params.append(doc_date_from)
        param_idx += 1

    if doc_date_to:
        conditions.append(f"document_date < ${param_idx}::date + 1")
        params.append(doc_date_to)
        param_idx += 1

    if min_pages:
        conditions.append(f"page_count >= ${param_idx}")
        params.append(min_pages)
        param_idx += 1

    if max_pages:
        conditions.append(f"page_count <= ${param_idx}")
        params.append(max_pages)
        param_idx += 1

    where_clause = " AND ".join(conditions) if conditions else "TRUE"

    # Build ranking expression.
    if q:
        q_param = f"${params.index(q) + 1}"
        rank_expr = f"""GREATEST(
            ts_rank(
                setweight(to_tsvector('simple', coalesce(file_name, '')), 'A') ||
                setweight(to_tsvector('simple', coalesce(metadata_text, '')), 'B') ||
                setweight(to_tsvector('simple', coalesce(content, '')), 'C'),
                plainto_tsquery('simple', {q_param})
            ),
            similarity(file_name, {q_param}) * 0.5
        )"""
        snippet_expr = f"""ts_headline('simple', content, plainto_tsquery('simple', {q_param}),
            'StartSel=<mark>, StopSel=</mark>, MaxWords=30, MinWords=10, MaxFragments=1')"""
    else:
        rank_expr = "1.0"
        snippet_expr = "LEFT(content, 150)"

    # Count total matches.
    total_count = await pool.fetchval(
        f"SELECT COUNT(*) FROM search_index WHERE {where_clause}",
        *params,
    )

    # Fetch ranked results.
    limit_param = f"${param_idx}"
    offset_param = f"${param_idx + 1}"
    params.extend([size, offset])

    rows = await pool.fetch(
        f"""
        SELECT
            document_id,
            file_name,
            {snippet_expr} AS snippet,
            {rank_expr} AS score
        FROM search_index
        WHERE {where_clause}
        ORDER BY score DESC
        LIMIT {limit_param} OFFSET {offset_param}
        """,
        *params,
    )

    items = [
        SearchResultItem(
            document_id=str(row["document_id"]),
            file_name=row["file_name"],
            snippet=row["snippet"] or "",
            score=round(float(row["score"]), 4),
        )
        for row in rows
    ]

    logger.debug("search query=%r filters=%d returned %d/%d results",
                 q, len(conditions) - (1 if q else 0), len(items), total_count)

    return SearchResponse(
        query=q,
        items=items,
        total_count=total_count,
        page=page,
        size=size,
    )
