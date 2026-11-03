# Rust Distributed Orchestrator

A Rust workspace starter for a durable distributed job orchestration platform. The first milestone provides a REST API, PostgreSQL persistence, job lifecycle operations, and a clean foundation for scheduler and worker services.

## Features

- Axum REST API with health and job endpoints
- PostgreSQL persistence through SQLx
- Job creation, listing, retrieval, status updates, and cancellation
- UUID identifiers and UTC timestamps
- Structured tracing and environment-based configuration
- Database migration and Docker Compose development environment
- Workspace split into applications and reusable crates

## Requirements

- Rust 1.85+ (edition 2024)
- Docker + Docker Compose (recommended), or PostgreSQL 14+

## Quick start

```bash
cp .env.example .env
docker compose up -d postgres
cargo run -p api-server
```

The API listens on `http://127.0.0.1:8080` by default.

### API examples

Create a job:

```bash
curl -X POST http://127.0.0.1:8080/api/v1/jobs \
  -H 'content-type: application/json' \
  -d '{"kind":"send_email","payload":{"to":"dev@example.com"},"priority":5}'
```

List jobs:

```bash
curl 'http://127.0.0.1:8080/api/v1/jobs?limit=20&offset=0'
```

Get a job:

```bash
curl http://127.0.0.1:8080/api/v1/jobs/JOB_UUID
```

Cancel a pending job:

```bash
curl -X DELETE http://127.0.0.1:8080/api/v1/jobs/JOB_UUID
```

Update status:

```bash
curl -X PATCH http://127.0.0.1:8080/api/v1/jobs/JOB_UUID/status \
  -H 'content-type: application/json' \
  -d '{"status":"running"}'
```

Allowed statuses: `pending`, `scheduled`, `running`, `completed`, `failed`, `cancelled`.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The API requires a reachable PostgreSQL database at `DATABASE_URL`. `cargo test --workspace` includes database-independent unit tests; API/database behavior can be exercised with Docker Compose.

## Project layout

- `apps/api-server`: HTTP API
- `apps/scheduler`: scheduler process placeholder
- `apps/worker`: worker process placeholder
- `crates/domain`: core job model and lifecycle rules
- `crates/application`: application-level use cases
- `crates/infrastructure`: PostgreSQL repository
- `crates/job-queue`: queue abstractions
- `crates/scheduler-core`: scheduling primitives
- `crates/worker-runtime`: worker execution primitives
- `crates/common`: shared utilities

## Current scope and next milestones

This is an initial runnable foundation, not yet a complete distributed production system. Next steps are durable atomic job claiming, worker heartbeats, retry/backoff, idempotency, authentication, metrics, integration tests, and Kafka/Redis adapters.

## License

MIT. See [LICENSE](LICENSE).
