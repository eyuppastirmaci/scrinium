# document-service

Java / Spring Boot service that owns document metadata. It accepts uploads over
HTTP, persists each document with a PENDING status, and publishes a
`document.uploaded` event. It also consumes `document.processing.completed`
events and transitions the matching document to READY.

The code follows a hexagonal (ports and adapters) structure:

- `domain`: the `Document` aggregate, its invariants, and the inbound/outbound
  ports. No framework or infrastructure code here.
- `application`: use-case services that orchestrate the domain through the ports.
- `adapter`: the web controller, the Kafka publisher and listener, and the JDBC
  repository that implement the ports.

Event models under `events.generated` are code-generated at build time from the
JSON Schema files in `../contracts`. Generated sources live under `target/` and
are not committed.

## Requirements

- JDK 25
- The backing infrastructure running (see the root `docker-compose.yml`):
  PostgreSQL on `localhost:5432` and Kafka on `localhost:9092`.

## Run

```bash
./mvnw spring-boot:run
```

The service starts on port 8080 and applies its Flyway migrations on startup.

## Test

```bash
./mvnw test
```

The domain and application tests are pure and fast. The Spring context test
needs the backing infrastructure to be running.

## Quick check

```bash
curl -F "file=@some-file.pdf" http://localhost:8080/documents
```

Returns `202 Accepted` with the new document id and a PENDING status. Once
processing-service reports completion, the document becomes READY.