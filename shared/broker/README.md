# broker

Message broker abstraction for ASEZ 2.0. Currently wraps RabbitMQ via `amqprs`.

## Core Traits

| Trait | Description |
|-------|-------------|
| `BrokerAdapter` | Main broker trait: connect, open channel, declare queue, register consumer/publisher, shutdown |
| `Consumer<C>` | Consume messages, send ack/nack, consume with timeout |
| `Publisher<C>` | Publish a serializable message to a queue |

## RabbitMQ Adapter

The `rabbit` module implements all three traits for RabbitMQ:

- `RabbitAdapter` — connection and channel management
- `RabbitConsumer` — receives messages from a queue
- `RabbitPublisher` — publishes messages to a queue

## Error Handling

`BrokerError` covers connection failures, serialization/deserialization errors, missing senders (`NoSenders`), and timeout (`WaitingTooLong`).

## Retry

`RetryArgs` configures exponential/fixed retry logic passed to `BrokerAdapter::connect`.

## Examples

See [`shared/broker/examples/`](examples/) for usage examples.
