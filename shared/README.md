# Shared Layer

Reusable crates shared across all ASEZ 2.0 microservices.

## Crate List

| Crate | Description |
|-------|-------------|
| [`shared-essential`](shared-essential/) | DDD-layer scaffolding and inter-service communication contracts (DTOs, domain types, message formatting) |
| [`broker`](broker/) | Message broker abstraction; RabbitMQ adapter via `amqprs` |
| [`rabbit-services`](rabbit-services/) | Queue name constants and `AsezRabbitRouting` enum for all system queues |
| [`shared-db`](shared-db/) | PostgreSQL connection pool helpers and `DbItem`/`DbAdaptor` traits built on `sqlx` |
| [`shared-db-derive`](shared-db-derive/) | Derive macros: `DbItem`, `DbAdaptor`, `DbItemExt`, `DbUpsert`, `DbVersioned`, `DbEnum` |
| [`table-entities`](table-entities/) | Database table entity definitions (plans, amendments, master data, TCP, scheduler, …) |
| [`env-setup`](env-setup/) | Environment variable constants and typed config structs for all services |
| [`trace-setup`](trace-setup/) | Shared tracing/logging initialisation (`TracingKind`) |
| [`igg-tracing`](igg-tracing/) | CEF (Common Event Format / Arclight) logger for security audit events |
| [`http-middleware`](http-middleware/) | actix-web middleware: session auth, RabbitMQ properties, tracing fields, domain ID extraction |
| [`format-tools`](format-tools/) | `numeric_format!` macro (grammatical number selection) and `fomat!` macro (custom delimiters) |
| [`fieldname-access`](fieldname-access/) | `FieldnameAccess` derive macro for runtime struct field access by name |
| [`monolith-service`](monolith-service/) | HTTP client for the GPI monolith REST API |
| [`testing`](testing/) | Test harness infrastructure: `TestHarness` trait, `id_pool`, database helpers |
| [`macros`](macros/) | `#[test]` attribute macro for async tests with `TestHarness` parameters |
