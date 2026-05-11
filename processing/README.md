# Processing Service

Core business logic service for purchase plans (PPZ) and contract amendments (DS).
Operates exclusively through RabbitMQ — no business HTTP API is exposed.

## How to Run

1. Start PostgreSQL where the database will run.
2. Start RabbitMQ.
3. Build and run the service:

```bash
cargo build --release
SRV_THREAD_COUNT=7 RABBITMQ_VHOST=localhost ... ./target/release/processing
```

## RabbitMQ Queues

### Consumed

| Queue | Description |
|-------|-------------|
| `processing` | General processing requests (plan operations from other services) |
| `plans` | Incoming PPZ from the monolith (legacy flow) |

### Declared (auxiliary)

| Queue | Description |
|-------|-------------|
| `contract_amendment` | Incoming DS from the monolith |

### Published to

| Queue | Description |
|-------|-------------|
| `plans_source` | Outbound PPZ/DS sent back to the monolith |

Sending to `plans_source` is disabled when the environment variable
`PROCESSING_NO_SEND_TO_PLANNING` is set to any non-`null` value.

## HTTP Endpoints

Processing exposes only monitoring endpoints (no business logic via HTTP):

| Method | Path | Description |
|--------|------|-------------|
| GET | `/monitoring/test` | Health check — returns `"Processing is alive"` |
| GET | `/monitoring/config` | Server configuration dump |

## Configuration

All settings come from environment variables. See [`shared/env-setup/README.md`](../shared/env-setup/README.md) for the full list.

Key variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `SRV_THREAD_COUNT` | `4` | Async worker thread count (minimum 4) |
| `PROCESSING_NO_SEND_TO_PLANNING` | — | Set to any non-null value to disable sending to monolith |
| `RABBITMQ_VHOST` | — | RabbitMQ host |
| `POSTGRES_VHOST` | — | PostgreSQL host |

## Additional Binary

The workspace also contains `sql-testing-tool` — see [`testing-tool/README.md`](testing-tool/README.md).
