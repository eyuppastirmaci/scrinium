# search-service

Python FastAPI service that provides full-text search over the Scrinium document
archive. It consumes Kafka events to build and maintain a search index, and
exposes a search endpoint for the web UI.

## Requirements

- Python 3.12+
- The backing infrastructure running (see the root `docker-compose.yml`): Kafka
  on `localhost:9092` and the search PostgreSQL database on `localhost:5434`.

## Setup

### 1. Start the search database

From the repository root:

```bash
docker compose --profile infra up -d postgres-search
```

### 2. Create the virtual environment and install dependencies

```bash
cd search-service
python3 -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
```

On Windows, activate the venv with `.venv\Scripts\activate` instead.

### 3. Configure environment

```bash
cp .env.example .env
```

Supported variables:

- `SEARCH_DATABASE_URL` — PostgreSQL connection string (default: `postgresql://scrinium:scrinium@localhost:5434/search`)
- `SEARCH_KAFKA_BROKERS` — Kafka bootstrap servers (default: `localhost:9092`)
- `SEARCH_KAFKA_GROUP_ID` — Kafka consumer group (default: `search-service`)
- `SEARCH_HTTP_HOST` — HTTP bind address (default: `127.0.0.1`)
- `SEARCH_HTTP_PORT` — HTTP port (default: `8092`)
- `SEARCH_LOG_LEVEL` — Log level: DEBUG, INFO, WARNING, ERROR (default: `INFO`)

### 4. Run

```bash
uvicorn app.main:app --host 127.0.0.1 --port 8092 --reload
```

The service applies database migrations on startup (creates the `search_index`
table with GIN full-text and trigram indexes), starts consuming Kafka events,
and exposes endpoints at:

- `GET /health` — health check
- `GET /search?q=...&page=0&size=20` — full-text search with ranking and snippets

## Pinned dependencies

`requirements.txt` contains pinned versions for reproducible installs:

```bash
pip install -r requirements.txt
```

To update after adding a dependency to `pyproject.toml`:

```bash
pip install -e ".[dev]"
pip freeze --exclude-editable > requirements.txt
```
