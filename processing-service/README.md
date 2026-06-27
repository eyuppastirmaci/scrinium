# processing-service

Rust service that processes documents asynchronously. It consumes
`document.uploaded` events, processes each document, and publishes a
`document.processing.completed` event back.

In the current phase processing is a no-op: the service proves the end-to-end
event loop without doing real work yet. Actual content processing (PDF rendering
and OCR) comes in a later phase.

The code follows a lightweight hexagonal structure:

- `domain`: the `EventPublisher` port and domain types. No Kafka or serialization
  code here.
- `application`: the use-case that handles an inbound event and decides whether
  the offset may be committed.
- `adapter`: the Kafka consumer setup and the `EventPublisher` implementation.
- `contract`: serde structs that mirror the JSON Schema in `../contracts`.
- `main`: the composition root that wires the adapters into the use-case and runs
  the receive loop.

Offsets are committed only after a result is published, giving at-least-once
delivery. Duplicate deliveries are harmless because the READY transition on the
document-service side is a conditional, idempotent update.

## Requirements

- A recent stable Rust toolchain (install via rustup).
- **CMake**, required to build the bundled `librdkafka` from source on all
  platforms. On Windows it is mandatory for local development, alongside the
  MSVC C++ build tools. Install CMake with `winget install Kitware.CMake` or from
  cmake.org.
- The backing infrastructure running (see the root `docker-compose.yml`): Kafka
  on `localhost:9092` and the processing PostgreSQL database on
  `localhost:5433`.
- Optional: `PROCESSING_DATABASE_URL` to override the default local database URL
  (`postgres://scrinium:scrinium@localhost:5433/processing`).

## Environment

Copy the example environment file and adjust local values as needed:

```bash
cp .env.example .env
```

On Windows PowerShell:

```powershell
Copy-Item .env.example .env
```

The service loads `.env` automatically when present. Keep `.env` local; it is
ignored by Git. The checked-in `.env.example` documents the supported variables:

- `PROCESSING_KAFKA_BROKERS`
- `PROCESSING_KAFKA_IN_TOPIC`
- `PROCESSING_KAFKA_GROUP_ID`
- `PROCESSING_DATABASE_URL`
- `PROCESSING_DB_MAX_CONNECTIONS`
- `PROCESSING_STORAGE_ENDPOINT`
- `PROCESSING_STORAGE_ACCESS_KEY`
- `PROCESSING_STORAGE_SECRET_KEY`
- `PROCESSING_STORAGE_BUCKET`
- `PROCESSING_TESSERACT_PATH`
- `PROCESSING_TESSERACT_LANGUAGES`
- `PROCESSING_HTTP_ADDR`

If Tesseract is available on `PATH`, leave `PROCESSING_TESSERACT_PATH=tesseract`.
Otherwise set it to the local executable path.

## Run

```bash
cargo run
```

The first build is slow because `librdkafka` is compiled from source; later
builds are fast. The service connects to Kafka, subscribes to
`document.uploaded`, applies its SQLx database migrations on startup, and logs
each event it receives and publishes.
