# igg-tracing

"Inline Group Gazprom Tracing" — a library for recording security-relevant events in CEF (Common Event Format) Arclight format. It ports an existing solution from inside the client's network; the client logs exclusively in this format, so the library is also needed for development outside that network.

## Usage Guide

A complete example is in `igg_tracing_integration_test` in `shared/igg-tracing/tests/lib.rs`.

The tracer is configured via `ServiceDescription`. Fields can be taken from a config file or any other source.

```rust
let service = ServiceDescription {
    vendor: "Super Vendor".to_owned(),
    name: "Super Service".to_owned(),
    version: "1.2.3".to_owned(),
    host: "127.0.0.1".parse().unwrap(),
};
```

A `CEFTracingLayer` is then created and inserted into the tracing registry.

```rust
let (cef_layer, _guard) = service.into_guarded_cef_layer(
    dir.path(),
    file_name,
    Rotation::NEVER,
    &["insert"].as_slice(),
    tz,
);
let _sub = tracing_subscriber::Registry::default().with(cef_layer).init();
```

Important notes:

- The `AppenderGuard` variable must **not** be named `_` — the compiler will drop it immediately and nothing will be written.
- The fourth argument to `into_guarded_cef_layer` is the list of event `kinds` to record. In the example above, `kinds == &["insert"]`, so only events where `kind = "insert"` will be written.

Once a span is open you can emit events. Field types must be exact:

- `u16` for `"source_port"`.
- `&str` for all other fields.

Using any other type will route values through `record_debug` and they will not be formatted correctly. This matches the original IGP implementation.

```rust
let span = tracing::span!(
    tracing::Level::TRACE,
    "test-span",
    "uri" = "/v1/insert",
    "user_agent" = "Bond",
    "user_code" = "007",
    "source_ip" = "localhost",
    // Must be u16, otherwise goes to debug.
    "source_port" = 3000u16,
    "request_id" = "some-uuid",
);
let _span_guard = span.enter();
```

Every event must have a `kind` field, otherwise it is ignored. The value must match one of those passed to `into_guarded_cef_layer`.

Preferred form:

```rust
tracing::trace!(kind = "insert", "Mr Hyde");
```

Backwards-compatible form:

```rust
tracing::trace!(kind = "insert", id = "Mr Hyde");
```

The preferred form also supports format strings:

```rust
tracing::trace!(kind = "insert", "Mr {}", "Hyde");
```

## Output Format

Each line written to the log file looks like:

```
Apr 12 10:58:51 127.0.0.1 CEF:0|Super Vendor|Super Service|1.2.3|Mr Hyde|Mr Hyde|1|suser=007 src=localhost spt=3000 request=/v1/insert requestClientApplication=Bond dpid=370096|
```

Breakdown:

- `127.0.0.1` and `Super Vendor|Super Service|1.2.3` come from `ServiceDescription` and are constant.
- `suser=007 src=localhost spt=3000 request=/v1/insert requestClientApplication=Bond` come from the span.
- `Mr Hyde|Mr Hyde` comes from the individual event.
- `dpid` and the timestamp are generated automatically.
