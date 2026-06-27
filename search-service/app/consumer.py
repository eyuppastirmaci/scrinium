"""Kafka consumer that listens to processing and deletion events and updates the search index."""

import json
import logging
import uuid

import asyncpg
from aiokafka import AIOKafkaConsumer

from app.indexer import upsert_document, delete_document, build_content_text, build_metadata_text

logger = logging.getLogger(__name__)

TOPICS = [
    "document.processing.completed",
    "document.deleted",
]


async def start_consumer(brokers: str, group_id: str, pool: asyncpg.Pool) -> None:
    """Start the Kafka consumer loop and process messages indefinitely.

    Subscribes to document processing and deletion topics. Each message
    is dispatched to handle_message which updates the search index.
    Runs until cancelled.

    Args:
        brokers: Kafka bootstrap servers (e.g. "localhost:9092").
        group_id: Kafka consumer group id for offset tracking.
        pool: asyncpg connection pool for search index writes.
    """
    consumer = AIOKafkaConsumer(
        *TOPICS,
        bootstrap_servers=brokers,
        group_id=group_id,
        auto_offset_reset="earliest",
        enable_auto_commit=True,
        value_deserializer=lambda v: json.loads(v.decode("utf-8")),
    )

    await consumer.start()
    logger.info("listening on topics: %s", ", ".join(TOPICS))

    try:
        async for message in consumer:
            try:
                await handle_message(pool, message.value)
            except Exception:
                logger.exception("error handling message")
    finally:
        await consumer.stop()


async def handle_message(pool: asyncpg.Pool, event: dict) -> None:
    """Dispatch a single Kafka event to the appropriate index operation.

    Supported event types:
    - document.processing.completed: upserts the document into the search index
      with file name, extracted text, and metadata.
    - document.deleted: removes the document from the search index.

    Unknown event types and events without a documentId are silently skipped.

    Args:
        pool: asyncpg connection pool.
        event: Deserialized Kafka message value (the full event envelope).
    """
    event_type: str = event.get("type", "")
    payload: dict = event.get("payload", {})
    document_id: str | None = payload.get("documentId")

    if not document_id:
        logger.warning("skipping event without documentId: %s", event_type)
        return

    doc_uuid = uuid.UUID(document_id)

    if event_type == "document.processing.completed":
        file_name: str = payload.get("fileName", "")
        pages: list[dict] = payload.get("pages", [])
        metadata: dict = payload.get("metadata", {})

        content = build_content_text(pages)
        metadata_text = build_metadata_text(metadata)

        await upsert_document(pool, doc_uuid, file_name, content, metadata_text)
        logger.info("indexed document %s (%d pages, %d chars)", document_id, len(pages), len(content))

    elif event_type == "document.deleted":
        await delete_document(pool, doc_uuid)
        logger.info("removed document %s from search index", document_id)
