# Scrinium

Scrinium is a small, self-hosted document archive built as a learning project to
explore a polyglot, event-driven microservice architecture. Users upload
documents; the system stores them, processes them asynchronously, and lets users
browse, search, and eventually ask questions about what they have uploaded.

The goal is not to build the most feature-rich document manager, but to practice
building a clean, well-bounded system: services that communicate through a
versioned event contract over Kafka, each owning its own database, each designed
with Domain-Driven Design and a hexagonal (ports and adapters) structure. The
same structure is meant to make later, more ambitious features possible by adding
new adapters rather than rewriting the core.

## Architecture

This is a monorepo containing several services:

- **document-service** (Java / Spring Boot): accepts uploads, owns document
  metadata, and publishes and consumes domain events.
- **processing-service** (Rust): consumes upload events and processes documents
  asynchronously.
- **contracts**: language-neutral JSON Schema definitions of the events exchanged
  between services. Both services generate or mirror their models from these.

Backing infrastructure (Kafka in KRaft mode, PostgreSQL, Redis, MinIO) runs via
Docker Compose. Services communicate asynchronously through Kafka; there is no
direct service-to-service calling.

## Getting started

During development the backing infrastructure runs in Docker, while the services
are run locally against it. (A fully containerized setup will come later.)

### 1. Start the infrastructure

From the repository root:

```bash
docker compose --profile infra up -d
```

This starts Kafka, two PostgreSQL instances, Redis, and MinIO. Check that they
are healthy with `docker compose --profile infra ps`.

### 2. Run document-service (Java / Spring Boot)

Requires JDK 25.

```bash
cd document-service
./mvnw spring-boot:run
```

It starts on port 8080 and applies its database migrations on startup.

### 3. Run processing-service (Rust)

Requires a stable Rust toolchain and CMake (used to build the bundled Kafka
client). On Windows, the MSVC C++ build tools are needed as well.

```bash
cd processing-service
cargo run
```

The first build is slow because the native Kafka client is compiled from source;
later builds are fast.

### 4. Try it

```bash
curl -F "file=@some-file.pdf" http://localhost:8080/documents
```

The document is created with a PENDING status; once processing-service reports
completion, it transitions to READY.

## Current features

- Upload a document over HTTP; its metadata is persisted with a PENDING status.
- An upload event is published and consumed across the Java and Rust services.
- Processing completion is reported back, transitioning the document to READY.
- The full event loop is idempotent and follows at-least-once delivery.

## Planned features

Near term, to make Scrinium a usable file manager:

- Real file storage in object storage (MinIO) instead of discarding uploaded
  bytes.
- Read endpoints to list and fetch documents and their status.
- A web UI for uploading and browsing documents, with live processing progress.
- An API gateway as a single entry point in front of the services.
- Text extraction (OCR for scans, direct extraction for digital PDFs) and
  full-text search over document content.

Longer term, to make it intelligent:

- Embeddings and semantic search, so a user can search by meaning rather than
  exact words (for example, "the most expensive bill from last year").
- A retrieval-augmented (RAG) assistant: ask a question in natural language, and
  the system finds the relevant documents and answers with an LLM, citing its
  sources.

These later features are intentionally deferred. The point of the current work is
to get the foundations right so they can be added cleanly when the time comes.

## Status

Early development. The core end-to-end flow works; most user-facing features are
still ahead.