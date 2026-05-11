# http-middleware

actix-web middleware stack used across services.

Typical web service setup:

```rust
let business_scope = web::scope("/v1")
    .wrap(DomainIDsTransform)
    .wrap(AsezSessionWatcher)
    .wrap(TracingLogger::<ServiceRootSpanBuilder>::new())
    .wrap(AsezTracingFields)
    .wrap(DefaultRabbitProperties)
    .service(...)
    ...
```

## [`DefaultRabbitProperties`](src/rabbit.rs)

Attaches default RabbitMQ adapter properties (`AsezRabbitProperties`) to the request.

Other middleware can extend these properties by adding additional fields.
For example, `AsezSessionWatcher` adds the authenticated user ID.

Must be placed before any middleware that extends the adapter properties.

## [`AsezTracingFields`](src/tracing_fields.rs)

Attaches a tracing field collector (`AsezTracingFieldsCollection`) to the request.

Some fields are populated immediately from the request parameters. Other middleware
(`AsezSessionWatcher`, `DomainIDsService`) fill in the remaining fields and must
run after this one.

## [`AsezSessionWatcher`](src/login.rs)

Must be present on all services that receive requests from the frontend. It inspects
the `user_id` and the authorization cookie (`id`) and validates them against the TECH
database (`view-storage`) via RabbitMQ.

In early stages the cookie check can be disabled because the frontend may not yet send it.

If the environment variable `AUTH_WITHOUT_COOKIE` is set to any non-`null` value,
the cookie is not checked.

## [`DomainIDsTransform`](src/domain_ids.rs)

Extracts numeric and unique identifiers of domain objects from the request.

The extracted identifiers are added to `AsezTracingFieldsCollection` and to
`RootSpan` when such objects are present in the request.
