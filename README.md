# Microservices

A Rust workspace of microservices built on actix-web + RabbitMQ + PostgreSQL.

## Architecture

```
FE ──HTTP──► Microservice ──RabbitMQ──► processing ──RabbitMQ──► Microservice ──HTTP──► FE
```

Services communicate through RabbitMQ. HTTP interfaces serve as entry points from the frontend.
Business logic is isolated in shared crates following DDD principles.

## Services

| Service | Description |
|---------|-------------|
| `processing` | Core plan and contract amendment processing engine. RabbitMQ only (no HTTP). |
| `master-data-service` | Reference data (master data) service. HTTP + RabbitMQ. |
| `technical-commercial-proposal` | Technical-commercial proposals service. HTTP + RabbitMQ. |

## Shared Crates

| Crate | Purpose |
|-------|---------|
| `broker` | RabbitMQ abstraction (`amqprs`) |
| `env-setup` | Unified configuration parser from environment variables |
| `shared-db` | PostgreSQL access (`sqlx`), table abstractions |
| `shared-db-derive` | Derive macros for `shared-db` |
| `shared-essential` | DDD domain: entities, DTOs, inter-service contracts |
| `table-entities` | Database tables and entities |
| `rabbit-services` | High-level service calls over RabbitMQ |
| `http-middleware` | actix-web middleware: sessions, tracing, CORS |
| `igg-tracing` | CEF logging in Arclight format |
| `trace-setup` | tracing-subscriber initialization |
| `monolith-service` | Monolith integration (HTTP driver) |
| `fieldname-access` | Runtime access to entity fields by name |
| `format-tools` | Numeric value formatting |
| `macros` | Procedural macros |
| `testing` | Test utilities |

## Stack

- **Language:** Rust 1.89.0 (pinned in `rust-toolchain`)
- **HTTP:** actix-web 4
- **Async:** tokio 1.18
- **Database:** PostgreSQL via sqlx 0.5
- **Message broker:** RabbitMQ via amqprs 1.5
- **Tracing:** tracing + igg-tracing (CEF/Arclight)

## Building

```bash
cargo build
```

For the production profile:

```bash
cargo build --release
```

## Configuration

All services are configured via environment variables. The full list is documented in [`shared/env-setup/README.md`](shared/env-setup/README.md).

Minimum required variables:

```bash
# RabbitMQ
RABBITMQ_HOST=localhost
RABBITMQ_VHOST=/
RABBITMQ_PORT=5672
RABBITMQ_USERNAME=guest
RABBITMQ_PASSWORD=guest

# PostgreSQL
POSTGRES_VHOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=postgres
POSTGRES_USER=postgres
POSTGRES_PASSWORD=changeme

# Server
SRV_PORT=3000

# Logging: 1=CEF, 2=normal, anything else=disabled
LOGGER_MODE=2
```

## Workspace Layout

```
.
├── processing/                     # Plan processing service (RabbitMQ only)
├── master-data-service/            # Reference data service (HTTP + RabbitMQ)
├── technical-commercial-proposal/  # TCP service (HTTP + RabbitMQ)
└── shared/
    ├── broker/              # RabbitMQ abstraction
    ├── env-setup/           # Configuration parser
    ├── shared-db/           # PostgreSQL abstraction
    ├── shared-db-derive/    # Derive macros for shared-db
    ├── shared-essential/    # DDD domain and contracts
    ├── table-entities/      # Database table entities
    ├── rabbit-services/     # Queue constants and routing enum
    ├── http-middleware/     # actix-web middleware
    ├── igg-tracing/         # CEF logging
    ├── trace-setup/         # Tracing initialisation
    ├── format-tools/        # numeric_format! and fomat! macros
    ├── fieldname-access/    # Runtime field access by name
    ├── monolith-service/    # Monolith HTTP client
    ├── macros/              # Procedural macros
    └── testing/             # Test harness utilities
```
