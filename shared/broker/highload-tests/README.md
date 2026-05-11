# Broker Highload Tests

Tests for the `broker` crate under high load and edge-case conditions. RabbitMQ must be running and configured before you start.

Do not run the scripts in parallel — they share the same queue.

# TODO

Improve code organisation: the same listener can be reused. Also extract `test_state` as a module so all binaries can share it.
