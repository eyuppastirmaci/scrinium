"""Database connection pool and schema migration for the search index."""

import logging

import asyncpg

logger = logging.getLogger(__name__)

_pool: asyncpg.Pool | None = None


async def init_pool(database_url: str) -> asyncpg.Pool:
    global _pool
    _pool = await asyncpg.create_pool(database_url, min_size=2, max_size=10)
    logger.info("connection pool created")
    return _pool


async def close_pool() -> None:
    global _pool
    if _pool:
        await _pool.close()
        _pool = None
        logger.info("connection pool closed")


def get_pool() -> asyncpg.Pool:
    assert _pool is not None, "database pool not initialized"
    return _pool


async def run_migrations(pool: asyncpg.Pool) -> None:
    async with pool.acquire() as conn:
        await conn.execute("CREATE EXTENSION IF NOT EXISTS pg_trgm")

        await conn.execute("""
            CREATE TABLE IF NOT EXISTS search_index (
                document_id UUID PRIMARY KEY,
                file_name TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                metadata_text TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )
        """)

        exists = await conn.fetchval("""
            SELECT 1 FROM pg_indexes
            WHERE indexname = 'search_index_fts'
        """)
        if not exists:
            await conn.execute("""
                CREATE INDEX search_index_fts ON search_index
                USING GIN (
                    (
                        setweight(to_tsvector('simple', coalesce(file_name, '')), 'A') ||
                        setweight(to_tsvector('simple', coalesce(metadata_text, '')), 'B') ||
                        setweight(to_tsvector('simple', coalesce(content, '')), 'C')
                    )
                )
            """)

        exists = await conn.fetchval("""
            SELECT 1 FROM pg_indexes
            WHERE indexname = 'search_index_trgm'
        """)
        if not exists:
            await conn.execute("""
                CREATE INDEX search_index_trgm ON search_index
                USING GIN (file_name gin_trgm_ops)
            """)

    logger.info("migrations applied")
