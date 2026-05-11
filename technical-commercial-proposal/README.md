# Technical-Commercial Proposal (TCP) Service

Service for managing price information requests (ZCI) and technical-commercial proposals (TCP/TKP).
Exposes both an HTTP API for frontend clients and a RabbitMQ interface for inter-service events.

## RabbitMQ

### Consumed

| Queue (`routing.rs` constant) | Queue name | Description |
|-------------------------------|------------|-------------|
| `TCP_QUEUE` | `tcp_action` | Inbound actions from partner services |

#### Handled message types on `tcp_action`

| Type | Handler |
|------|---------|
| `CommercialOfferRequestConfirmation` | Confirm a commercial offer request |
| `CommercialOfferResponse` | Record a commercial offer response |
| `CommercialOfferAddDocResponse` | Record an additional document response |

### Outbound calls

The service makes RPC calls to the `processing` service queue for plan-related data.

## HTTP API

All business routes are under `/v1/` and require session authentication (`AsezSessionWatcher`).

### Price information requests

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/create_price_information_request` | Create a new ZCI |
| POST | `/v1/get/request_price_info_list/` | List ZCIs |
| POST | `/v1/get/request_price_info_detail/` | ZCI detail |
| POST | `/v1/update/request_price_info/` | Update ZCI |
| DELETE | `/v1/delete/request_price_info/` | Delete ZCI |
| POST | `/v1/action/request_price_info_close/` | Close ZCI |
| POST | `/v1/action/request_price_info_complete/` | Complete ZCI |
| POST | `/v1/action/request_price_info_publication/` | Publish ZCI |
| POST | `/v1/pre_request/request_price_info_close/` | Pre-validate ZCI close |
| POST | `/v1/check/request_price_info/` | Validate ZCI |
| POST | `/v1/get_price_information_request_by_plan_uuid/{uuid}/` | Get ZCI by plan UUID |
| POST | `/v1/get_price_information_request_by_plan_uuid_vec/` | Get ZCIs by plan UUID list |
| POST | `/v1/send_for_proposal_price_request/` | Send ZCI to partners for proposals |
| POST | `/v1/complete_price_request/` | Complete ZCI price collection |

### Proposals (TKP)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/get/proposal_detail/` | Proposal detail |
| POST | `/v1/get/proposal_list_by_object_id/` | Proposals by plan object ID |
| POST | `/v1/get/proposal_items_for_pricing/` | Proposal items for pricing |
| POST | `/v1/get/technical_commercial_proposal/` | Full TKP data |
| POST | `/v1/get_tkp_by_request_uuid/{uuid}/` | TKP by ZCI UUID |
| POST | `/v1/get_tkp_by_request_uuid_vec/` | TKPs by ZCI UUID list |
| POST | `/v1/update/proposal/` | Update proposal |
| POST | `/v1/action/proposal_approve/` | Approve proposal |
| POST | `/v1/action/proposal_apply_pricing_consider/` | Apply pricing consideration |
| POST | `/v1/tkp_reject/` | Reject TKP |
| POST | `/v1/tkp_verified/` | Mark TKP as verified |

### Partners and organizations

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/check/add_partner/` | Validate adding a partner |
| POST | `/v1/check/delete_partner/` | Validate removing a partner |
| GET | `/v1/get/organizations/{uuid_subject}/` | Organizations for a subject |
| POST | `/v1/update/organizations/` | Update organizations |
| POST | `/v1/action/organizations_remove/` | Remove organizations |
| POST | `/v1/pre_request/organization_question/` | Pre-validate organization question |

### Purchasing subjects

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/get/purchasing_subject_by_group_uuid/{uuid}/` | Subjects by group UUID |
| GET | `/v1/get/purchasing_subject_group/` | All subject groups |
| POST | `/v1/update/purchasing_subject_group/` | Update subject group |
| POST | `/v1/update/purchasing_subject/` | Update purchasing subject |
| POST | `/v1/action/purchasing_subject_group_remove/` | Remove subject group |
| POST | `/v1/action/purchasing_subject_remove/` | Remove purchasing subject |

### Reports and export

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/export/table/` | Export table data |
| POST | `/v1/create_report/` | Generate price request report |

### Monitoring

| Method | Path | Description |
|--------|------|-------------|
| GET | `/monitoring/test` | Health check — returns `"Technical Commercial Proposal is alive"` |
| GET | `/monitoring/config` | Server configuration dump |

## Configuration

See [`shared/env-setup/README.md`](../shared/env-setup/README.md) for all environment variables.
Requires `RABBITMQ_*`, `POSTGRES_*`, and `SRV_PORT` at minimum.
