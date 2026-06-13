# Event contracts

Language-neutral source of truth for the events exchanged between services.
Both `document-service` (Java) and `processing-service` (Rust) generate their
event models from these schemas. The schema is the contract; the generated code
is just a convenience artifact.

## Envelope

Every event shares the same top-level shape:

| Field       | Type            | Notes                                              |
|-------------|-----------------|----------------------------------------------------|
| `id`        | UUID string     | Unique per message. Used for consumer idempotency. |
| `type`      | string          | Event type discriminator, e.g. `document.uploaded`.|
| `version`   | integer (>= 1)  | Schema major version; matches the `.vN` filename.  |
| `timestamp` | date-time       | RFC 3339 / ISO-8601, UTC.                           |
| `payload`   | object          | Event-specific, typed per schema.                  |

## Conventions

- **Field naming:** camelCase on the wire (`documentId`, not `document_id`).
  Java keeps these as-is; Rust renames to snake_case fields while preserving the
  JSON names.
- **Identifiers:** UUIDs are transported as strings (`format: uuid`).
- **Timestamps:** always UTC, RFC 3339 (`2026-06-13T10:15:30Z`).
- **Forward compatibility:** `additionalProperties` is intentionally left open.
  Consumers must ignore unknown fields so producers can add fields without
  breaking older consumers:
  - Java: `FAIL_ON_UNKNOWN_PROPERTIES = false`
  - Rust: serde ignores unknown fields by default.
- **Versioning:** a breaking change means a new file (`*.v2.json`) and a bumped
  `version`. Old consumers keep reading the old version until migrated.

## Draft version

Schemas target **JSON Schema draft-07**. This is the common denominator both
code generators support reliably (jsonschema2pojo on the Java side, typify on
the Rust side). Do not bump the `$schema` draft without verifying both
generators still produce correct models.

## Codegen targets

- **Java (`document-service`):** `jsonschema2pojo` (Gradle/Maven plugin).
- **Rust (`processing-service`):** `typify` (via `build.rs`) or `schemars`.

## Kafka topics

| Event                            | Topic                            |
|----------------------------------|----------------------------------|
| `document.uploaded`              | `document.uploaded`              |
| `document.processing.completed`  | `document.processing.completed`  |

Messages are keyed by `documentId` to preserve per-document ordering if the
topic is ever partitioned.