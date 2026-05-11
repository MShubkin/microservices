# trace-setup

A shared tracing/logging setup library for ASEZ 2.0 services.

Kept separate from `shared-essential` to reduce compile-time coupling — it can be compiled in parallel with other crates.

## `TracingKind`

Controls which logging backend is active. Configured via `LOGGER_MODE` (see `env-setup`):

| Variant | `LOGGER_MODE` value | Description |
|---------|---------------------|-------------|
| `None` | any other value | Logging disabled |
| `Normal` | `2` / `"normal"` | Human-readable stdout (development) |
| `Cef` | `1` / `"cef"` | CEF format to stdout |
| `JsonCef { path }` | `3` / `"json_cef"` | CEF to file (`LOGGER_FILE`) + JSON Bunyan to stdout |

Call `TracingKind::initiate_log(service_name, host, port, access_ops)` to initialize tracing and obtain the `AppenderGuard`s that must be kept alive for the duration of the process.

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `ACCESS_OPS` | `["get", "insert", "update"]` | Standard CEF operation kinds |
| `SECURITY_OPS` | `["users"]` | Security audit operation kinds |

## Helper Functions

| Function | Description |
|----------|-------------|
| `new_cef(service, path, ops)` | Create a file-backed `CEFTracingLayer` |
| `new_cef_stdout(service, ops)` | Create a stdout-backed `CEFTracingLayer` |

## Dependencies

- [`igg-tracing`](../igg-tracing/) — CEF layer implementation
- `tracing-bunyan-formatter` — JSON log formatter
- `tracing-appender` — async file appender
