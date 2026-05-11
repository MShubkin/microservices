//! Модуль с таблицами из СК и АЦ, которые контролирует процессинг.
use super::*;

pub mod agenda;
pub mod agenda_item;
pub mod attachment;
pub mod ca_item;
pub mod commission_result;
pub mod contract_amendment;
pub mod estimated_commission_partner;
pub mod estimated_commission_settings;
pub mod executor_method;
pub mod favourite_plan;
pub mod field_histories;
pub mod object_route;
pub mod object_type;
pub mod partner_agenda_protocol;
pub mod partner_type;
pub mod partner_type_commission;
pub mod plan;
pub mod plan_item;
pub mod plan_retrospective;
pub mod price_analysis_user;
pub mod protocol;
pub mod protocol_item;
pub mod regulatory_deadline_price;
pub mod rel_agenda_protocol;
pub mod rel_agenda_protocol_item;
pub mod status_history;
pub mod status_object;
// specialized department entities
pub mod document_approver;
pub mod route_addep;
