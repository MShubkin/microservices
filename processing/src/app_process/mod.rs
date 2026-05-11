//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.
pub(crate) mod calls;
pub(crate) mod calls_legacy;
pub(super) mod common;
pub(crate) mod records;
pub(crate) mod sections;
pub(crate) mod validation;

pub use shared_essential::application::external;

#[cfg(test)]
pub(crate) mod tests;

// common
pub(crate) use calls::{
    export_data::*, get_attachments_meta::get_attachments_meta,
    get_plans::get_plans, get_plans_count::get_plans_count, update_plans::*,
};

// СК
pub(crate) use calls::estimated_commission;
pub(crate) use estimated_commission::{
    add_plans_agenda::{add_plans_agenda, pre_add_plans_agenda},
    add_plans_protocol::{add_plans_protocol, pre_add_plans_protocol},
    agenda_items_remove::*,
    agenda_remove::{
        action::action_agenda_remove, pre_request::pre_request_agenda_remove,
    },
    agenda_send::*,
    approve_plans::{action::*, pre_request::*},
    approve_protocol::*,
    assign_expert::*,
    cancel::*,
    change_commission_date::*,
    change_form::*,
    confirm_decision::confirm_decision,
    create_agenda::*,
    create_protocol::*,
    ec_get_sections_count::*,
    get_agenda_details::get_agenda_details,
    get_agenda_items_by_id_range::get_agenda_items_by_id_range,
    get_agenda_items_for_protocol_create::get_agenda_items_for_protocol_create,
    get_agenda_list::get_agenda_list,
    get_agenda_list_by_date::get_agenda_list_by_date,
    get_item_list::get_item_list,
    get_partners::get_partners,
    get_plans_with_last_agenda_item::get_plans_with_last_agenda_items,
    get_protocol_details::get_protocol_details,
    get_protocol_items_by_id_range::get_protocol_items_by_id_range,
    get_protocol_list::get_protocol_list,
    get_protocol_list_by_agenda::get_protocol_list_by_agenda,
    get_protocol_list_by_date::get_protocol_list_by_date,
    import_item_list_specific::*,
    protocol_agreement::{
        action::action_protocol_agreement,
        pre_request::pre_request_protocol_agreement,
    },
    remove_protocol::*,
    return_to_customer::*,
    return_to_expert::*,
    send_protocols_for_signing::*,
    transfer_plans_agenda::{pre_transfer_plans_agenda, transfer_plans_agenda},
    update_agenda::*,
    update_protocol::*,
};

// АЦ
pub(crate) use calls::price_analysis;
pub(crate) use price_analysis::{
    assign_expert_mass, export_specification, get_complete_contract_amendments,
    get_complete_plans, get_contract_amendment_version, get_plan_version,
    get_price_analysis_user, get_retrospective, import_specification,
    pa_approve_by_chief, pa_complete_lotting, pa_decline_by_chief,
    pa_documentation_checked, pa_get_sections_count, pa_pre_decline_by_chief,
    pa_pre_request_documentation, pa_pre_request_documents_for_expert,
    pa_pre_return_to_customer, pa_price_determined, pa_pricing_result,
    pa_request_documentation, pa_return_to_customer, pa_review_progress,
    pa_update_contract_amendment, pa_update_plan, pricing_report_commission_data,
    pricing_report_common_data, pricing_report_savings_data,
};

// Взаимодействие с АСЭЗ 1.0
pub(crate) use calls_legacy::amendment_insert_update::*;
pub(crate) use calls_legacy::plan_insert_update::*;
