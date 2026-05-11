# Master Data Service (NSI)

Reference data (master data) service for the ASEZ 2.0 system.
Exposes both an HTTP API for frontend clients and a RabbitMQ interface for inter-service communication.

## RabbitMQ Queues

### Consumed

| Queue (`routing.rs` constant) | Queue name | Description |
|-------------------------------|------------|-------------|
| `REQUEST_DICTIONARY_QUEUE` | `dictionary` | Fetch a reference dictionary |
| `MASTER_DATA_ACTION_QUEUE` | `master_data_action` | Perform an action on master data |
| `ROUTING_QUEUE` | `auto_routing` | Auto-routing assignments |

## HTTP API

All business routes are under `/v1/` and require session authentication (`AsezSessionWatcher`).

### Generic dictionary endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/{directory}/{id}/` | Get record by ID |
| POST | `/v1/{directory}/search_by_id/` | Search by list of IDs |
| POST | `/v1/{directory}/search/` | Full-text / filtered search |
| GET | `/v1/master_data/get_updates/{timestamp}/` | Incremental updates since timestamp |

### Hierarchy

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/get/hierarchy/{dictionary}/` | Hierarchical tree for a dictionary |

### Organizational structure

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/organizational_user_assignment/search/` | Search user assignments |
| POST | `/v1/organizational_user_assignment/search_by_id/` | Search user assignment by ID |
| POST | `/v1/organizational_structure/search/` | Search organizational structure |
| POST | `/v1/organizational_structure/search_by_id/` | Search org structure by ID |

### Plan cancellation reasons (`plan_reason_cancel`)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/plan_reason_cancel/get/item_list/` | List all items |
| POST | `/v1/plan_reason_cancel/get/detail/` | Item detail |
| POST | `/v1/plan_reason_cancel/search/` | Search |
| POST | `/v1/plan_reason_cancel/search_by_id/` | Search by ID |
| POST | `/v1/plan_reason_cancel/create/item/` | Create |
| POST | `/v1/plan_reason_cancel/update/item/` | Update |
| POST | `/v1/plan_reason_cancel/delete/item/` | Soft delete |
| POST | `/v1/plan_reason_cancel/restore/item/` | Restore |
| POST | `/v1/plan_reason_cancel/export/` | Export |

### Favourites

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/get/favorite_list/` | Get favourite items |
| POST | `/v1/create/favorite_item/` | Add to favourites |
| DELETE | `/v1/delete/favorite_item/` | Remove from favourites |

### Monitoring

| Method | Path | Description |
|--------|------|-------------|
| GET | `/monitoring/test` | Health check — returns `"Master-Data is alive"` |
| GET | `/monitoring/config` | Server configuration dump |

## Configuration

See [`shared/env-setup/README.md`](../shared/env-setup/README.md) for all environment variables.
Requires `RABBITMQ_*`, `POSTGRES_*`, and `SRV_PORT` at minimum.
