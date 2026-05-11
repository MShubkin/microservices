# Env Setup

The `env-setup` mini-library provides a standard environment variable layout and configuration structs for all services.

## Exported Types

| Type | Description |
|------|-------------|
| `EnvCfg` | Complete configuration struct (rabbit + server + postgres + email + logger) |
| `RabbitCfg` | RabbitMQ connection settings |
| `ServerCfg` | HTTP server settings |
| `PostgresCfg` | PostgreSQL connection settings |
| `MailerCfg` | Email/SMTP settings |
| `TracingCfg` | Logging/tracing mode |
| `PlanAddress` | Internal address of the plan server |
| `PlanningRestCfg` | Monolith REST base URL |
| `MonolithCfg` | Monolith base URL + technical user ID |
| `MDSCfg` | Master Data Service base URL |
| `PayloadConfig` | HTTP payload size limit |
| `JsonConfig` | HTTP JSON body size limit |
| `EnvError` | Unified error type for env parsing |

Each config struct has a `from_env()` constructor. `PostgresCfg` also provides `get_connection_string()`.

## Helper Functions

| Function | Description |
|----------|-------------|
| `var(name)` | Get a required env variable, errors if missing |
| `var_maybe(name)` | Get an optional env variable |
| `try_get(name, default)` | Get and parse, falling back to a default |
| `try_get_maybe(name)` | Get and parse into `Option<T>` |

## Environment Variable Reference

Not every service requires all variables.

```bash
# Logging mode.
# Accepted values:
#   1 / cef      — CEF format to stdout
#   2 / normal   — human-readable to stdout
#   3 / json_cef — CEF to LOGGER_FILE, JSON to stdout
#   anything else — logging disabled
LOGGER_MODE=2
LOGGER_DIR=logs
LOGGER_FILE=log.cef

# RabbitMQ
RABBITMQ_HOST=rcdevstand.inlinegroup.ru
RABBITMQ_VHOST=rcdevstand.inlinegroup.ru
RABBITMQ_PORT=5672
RABBITMQ_USERNAME=astra
RABBITMQ_PASSWORD=astra
RABBITMQ_RETRIES=10
RABBITMQ_INTERVAL_MS=500

# Internal address of the plan server
PLANDB_SRV_INNER_ADDR=127.0.0.1
PLANDB_SRV_INNER_PORT=3004

# HTTP server
SRV_HOST=0.0.0.0
SRV_PORT=3000
SRV_WORKERS=2
SRV_MAX_CONN_PER_WORKER=1000
SRV_BLOCKING_THREADS_PER_WORKER=2
SRV_THREAD_COUNT=7
SRV_PAYLOAD_LIMIT=262144   # bytes, default 256 KiB
SRV_JSON_LIMIT=2097152     # bytes, default 2 MiB

# PostgreSQL
POSTGRES_VHOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=postgres
POSTGRES_USER=postgres
POSTGRES_PASSWORD=changeme
POSTGRES_MIN_CONNECTIONS=1
POSTGRES_MAX_CONNECTIONS=4
POSTGRES_TIMEOUT_S=10
POSTGRES_CONNECTION_REFRESH_S=3600

# Email (for services that send mail)
EMAIL_LOGIN=stage_test@inlinegroup.ru
EMAIL_PASSWORD=whatever
EMAIL_SENDER=stage_test@inlinegroup.ru
EMAIL_SMTP_SERVER=rcdevstand.inlinegroup.ru
EMAIL_SMTP_PORT=25

# Monolith REST base URL
MONOLITH_BASE_URL=http://monolith.example.com
MONOLITH_TECH_USER_ID=1

# Master Data Service base URL
MASTER_DATA_BASE_URL=http://mds.example.com
```
