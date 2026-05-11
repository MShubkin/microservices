# shared-essential

Core shared library for the ASEZ 2.0 microservices. Contains DDD-layer components and inter-service communication contracts.

## Structure

The crate mirrors a standard DDD layout:

### `domain`

Business entities and table definitions. Re-exports:
- `asez2_tables` — plan/amendment table structs and traits
- `PlanOrAmendment` — discriminated union of plan and contract amendment
- `master_data` — master data domain types
- `enums` — shared domain enumerations

### `application`

Application-layer services and utilities:
- `message` — message formatting helpers, including `numeral_relation` (grammatical number selection)
- `records` — record status handling, validation rules, historian
- `commission` — estimated commission helpers
- `external` — integration with external services
- `routes` — routing utilities
- `validation` — shared validation logic

### `infrastructure`

Technical plumbing shared across services:
- `rabbit` — RabbitMQ connection and channel setup helpers
- `db` — database pool helpers
- `migration` — SQLx migration utilities
- `server_config` — HTTP server configuration
- `background` — background task utilities

### `presentation`

Shared HTTP layer components:
- `dto` — data transfer objects for all inter-service contracts, organized by service domain:
  - `processing`, `view_storage`, `master_data`, `notification`, `print_docs`, `integration`
  - `log_storage`, `scheduler`, `specialized_departments`, `technical_commercial_proposal`
  - `estimated_commission`, `price_analysis`
  - `general`, `export`, `import`, `value`
- `error` — shared HTTP error types
- `response_request` — standard request/response wrappers
