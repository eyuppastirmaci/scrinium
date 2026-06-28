# Scrinium

Scrinium is a self-hosted document archive that automatically processes your
uploads — extracting text via OCR, reading metadata, generating thumbnails — and
makes everything searchable. Upload PDFs and images through the web UI, watch
processing happen in real time, then browse, search, and preview your documents
from anywhere.

![Scrinium Demo](.github/assets/scrinium-demo.gif)

## Services

This is a monorepo containing five components:

### document-service (Java / Spring Boot)

The central service that owns document lifecycle. Accepts uploads, stores files in
MinIO, manages document status (PENDING → READY / FAILED), and serves all read
endpoints (metadata, extracted text, thumbnails, download). Consumes enriched
processing events from Kafka to persist processing results. Provides an SSE
endpoint for real-time processing progress via Redis Pub/Sub.

- **Port:** 8080
- **Database:** PostgreSQL on port 5432
- **Tech:** Java 25, Spring Boot, Spring Data Redis, Flyway, MinIO SDK

### processing-service (Rust)

Asynchronous document processor. Consumes `document.uploaded` events from Kafka,
processes each document (text extraction, OCR, metadata extraction, thumbnail
generation), and publishes enriched `document.processing.completed` events with
all results. Does not serve HTTP — pure event-driven processing.

- **Kafka consumer** with semaphore-based parallel processing (auto-detects CPU cores)
- **Database:** PostgreSQL on port 5433 (job tracking only)
- **Tech:** Rust, Tokio, rdkafka, SQLx, PDFium, Tesseract OCR, image crate

### search-service (Python / FastAPI)

Full-text search over the document archive. Consumes Kafka events to build and
maintain a search index. Provides a search endpoint with weighted ranking,
trigram similarity for typo tolerance, highlighted snippets, and filters.

- **Port:** 8092
- **Database:** PostgreSQL on port 5434
- **Tech:** Python 3.12, FastAPI, asyncpg, aiokafka, PostgreSQL tsvector + pg_trgm

### gateway-service (Java / Spring Cloud Gateway)

Single entry point for the web UI. Routes requests to document-service and
search-service. Proxies SSE streams for real-time progress.

- **Port:** 8090
- **Tech:** Java 25, Spring Cloud Gateway

### webui (Vue 3 / TypeScript)

Single-page application for uploading, browsing, searching, and previewing
documents. Features include drag-and-drop upload, grid/list views, document
detail with metadata/text/preview tabs, PDF.js viewer, search with filters,
real-time processing progress indicators, toast notifications, and an activity
dropdown.

- **Dev server:** port 5173
- **Tech:** Vue 3, TypeScript, Vite, PDF.js, Lucide Icons

### contracts

Language-neutral JSON Schema definitions of the events exchanged between services.

## Getting Started

### 1. Start the infrastructure

```bash
docker compose --profile infra up -d
```

This starts Kafka, three PostgreSQL instances (document, processing, search),
Redis, and MinIO. Verify with `docker compose --profile infra ps`.

| Service            | Port  |
|--------------------|-------|
| Kafka              | 9092  |
| PostgreSQL (doc)   | 5432  |
| PostgreSQL (proc)  | 5433  |
| PostgreSQL (search)| 5434  |
| Redis              | 6379  |
| MinIO API          | 9000  |
| MinIO Console      | 9001  |

### 2. Run document-service

Requires JDK 25.

```bash
cd document-service
./mvnw spring-boot:run
```

Starts on port 8080. Applies Flyway migrations on startup.

### 3. Run processing-service

Requires a stable Rust toolchain and CMake.

```bash
cd processing-service
cp .env.example .env       # edit paths for Tesseract and PDFium
cargo run
```

The first build is slow (native Kafka client compiled from source).

**Environment variables** (`.env`):

| Variable | Default | Description |
|----------|---------|-------------|
| `PROCESSING_KAFKA_BROKERS` | `localhost:9092` | Kafka bootstrap servers |
| `PROCESSING_DATABASE_URL` | `postgres://scrinium:scrinium@localhost:5433/processing` | PostgreSQL connection |
| `PROCESSING_STORAGE_ENDPOINT` | `http://localhost:9000` | MinIO endpoint |
| `PROCESSING_TESSERACT_PATH` | `tesseract` | Tesseract executable path |
| `PROCESSING_TESSERACT_LANGUAGES` | `tur+eng` | OCR languages |
| `PROCESSING_PDFIUM_PATH` | `./` | Directory containing pdfium.dll/libpdfium.so |
| `PROCESSING_REDIS_URL` | `redis://localhost:6379` | Redis for progress reporting |
| `PROCESSING_MAX_CONCURRENCY` | auto (cores-1)/2 | Max parallel document processing |

### 4. Run search-service

Requires Python 3.12+.

```bash
cd search-service
python3 -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
cp .env.example .env
pip install -e ".[dev]"
uvicorn app.main:app --host 127.0.0.1 --port 8092 --reload
```

**Environment variables** (`.env`):

| Variable | Default | Description |
|----------|---------|-------------|
| `SEARCH_DATABASE_URL` | `postgresql://scrinium:scrinium@localhost:5434/search` | PostgreSQL connection |
| `SEARCH_KAFKA_BROKERS` | `localhost:9092` | Kafka bootstrap servers |
| `SEARCH_LOG_LEVEL` | `INFO` | Log level (DEBUG/INFO/WARNING/ERROR) |

### 5. Run gateway-service

Requires JDK 25.

```bash
cd gateway-service
./mvnw spring-boot:run
```

Starts on port 8090. All API requests go through the gateway.

### 6. Run webui

Requires Node.js 18+.

```bash
cd webui
npm install
npm run dev
```

Opens at http://localhost:5173. The dev server proxies `/api` to the gateway on
port 8090.

### 7. Try it

Open http://localhost:5173, drag and drop files to upload. Watch real-time
processing progress, then browse, search, and preview your documents.

## Current Features

- **Upload:** Drag-and-drop or file picker, multiple files, duplicate detection
- **Processing:** PDF text extraction (digital + scanned via OCR), image OCR, metadata extraction, thumbnail generation
- **Parallel processing:** CPU-aware concurrency with semaphore control
- **Document detail:** Metadata tab, extracted text tab (copy/download), PDF.js preview, image preview
- **Thumbnails:** Auto-generated for PDFs and images, shown in document lists
- **Search:** Full-text search with weighted ranking, typo tolerance, highlighted snippets, filters (type, date, page count)
- **Real-time progress:** Redis Pub/Sub → SSE → circular progress indicators, toast notifications, activity dropdown
- **Retry:** Failed documents can be retried from the detail page
- **Delete:** Soft delete with full cleanup (DB records, search index, MinIO thumbnails)

## Planned Features

### Phase 1: Text-based semantic & hybrid search

- **Text embeddings over extracted text**: multilingual dense embeddings
  (e.g. BGE-M3) generated from OCR/extracted text at index time
- **Hybrid retrieval**: combine the existing full-text (keyword) search with
  dense semantic search via rank fusion (RRF), augmenting it rather than
  replacing it
- Semantic search in the UI: ranked results, highlighted snippets, filters
- Auto-tagging and categorization with an LLM (over extracted text)

### Phase 2: Vision-based embeddings & multimodal retrieval

High-level direction only; design to be detailed when this phase begins.

- **VLM-based page-image embeddings + visual search** over a multi-vector
  vector database (ColPali / ColQwen + Qdrant)
- **Vision-first RAG assistant**: retrieved page images fed directly to a
  generative VLM for Q&A
- **Hybrid text + vision retrieval**: fuse the Phase 1 text path with the
  vision path rather than discarding either
- Goal: handle visually-rich documents (tables, charts, diagrams, complex
  layouts, low-quality scans) that text/OCR retrieval misses

### Platform & operations

- Authentication and multi-user support
- Folder and collection organization
- Storage analytics dashboard
- Email-to-archive forwarding
- Document retention policies
- Export and backup
- Webhook integrations
- Fully containerized services