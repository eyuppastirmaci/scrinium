"""Search service entry point — bootstrap and lifespan management."""

import asyncio
import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI

from app.config import settings
from app.consumer import start_consumer
from app.db import init_pool, close_pool, run_migrations
from app.router import router

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    # Connect to PostgreSQL and apply schema migrations.
    pool = await init_pool(settings.database_url)
    await run_migrations(pool)
    logger.info("database ready")

    # Start event consumer as a background task so it runs alongside the HTTP server.
    consumer_task = asyncio.create_task(
        start_consumer(settings.kafka_brokers, settings.kafka_group_id, pool)
    )
    yield

    # Graceful shutdown: cancel the consumer and close the database pool.
    consumer_task.cancel()
    try:
        await consumer_task
    except asyncio.CancelledError:
        pass
    await close_pool()


app = FastAPI(title="Scrinium Search Service", lifespan=lifespan)
app.include_router(router)
