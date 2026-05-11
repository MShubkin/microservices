use shared_essential::{concat_slice, domain::*};

pub(super) const UPDATE_AGENDA_DETAILS: &str =
    "/rest/estimated_commission/v1/get/agenda_update";

pub(super) const AGENDA_UPDATE_FIELDS: &[&str] = &[
    EcAgenda::pricing_organization_unit_id,
    EcAgenda::meeting_date,
    EcAgenda::changed_at,
    EcAgenda::changed_by,
];

pub(super) const AGENDA_FIELDS: &[&str] = concat_slice!(
    AGENDA_UPDATE_FIELDS,
    [
        EcAgenda::created_at,
        EcAgenda::created_by,
        EcAgenda::id,
        EcAgenda::status_id,
        EcAgenda::uuid,
    ]
);

pub(super) const ITEM_FIELDS: &[&str] = &[
    EcAgendaItem::uuid,
    EcAgendaItem::source_uuid,
    EcAgendaItem::is_excluded,
    EcAgendaItem::reviewed_at,
];

/// Поля из базы данных, которые требуются для обновления позиций повестки:
/// - поля, которые мы берем из БД
/// - поля, требуемые для определения наличия изменений
///
/// См. [UpdateAgendaContext::new] и [has_changes]
pub(super) const AGENDA_OLD_ITEM_FIELDS: &[&str] = concat_slice!(
    ITEM_FIELDS,
    [
        EcAgendaItem::created_at,
        EcAgendaItem::created_by,
        EcAgendaItem::number,
        EcAgendaItem::is_removed,
        EcAgendaItem::sum_excluded_vat,
        EcAgendaItem::pricing_sum_excluded_vat,
        EcAgendaItem::is_registered_by_d647,
    ]
);

pub(super) const AGENDA_UPDATE_ITEM_FIELDS: &[&str] = &[
    // uuid позиции, ппз/дс и повестки не меняются
    EcAgendaItem::number,
    EcAgendaItem::is_registered_by_d647,
    EcAgendaItem::is_excluded,
    EcAgendaItem::is_removed,
    EcAgendaItem::reviewed_at,
    EcAgendaItem::sum_excluded_vat,
    EcAgendaItem::pricing_sum_excluded_vat,
];

pub(super) const RETURN_PLAN_FIELDS: &[&str] = &[
    "plan_id", // переименованное поле.
    Plan::customer_id,
    Plan::number_customer,
    Plan::status_id,
    Plan::supplier_id,
    Plan::contract_subject,
    Plan::currency_id,
    Plan::sum_excluded_vat,
    Plan::pricing_sum_excluded_vat,
    Plan::pricing_expert_id,
    Plan::pricing_resume,
    Plan::section_id,
];

pub(super) const PLAN_FIELDS: &[&str] =
    concat_slice!(AGENDA_UPDATE_FIELDS, [Plan::uuid]);

pub(super) const PARTNER_FIELDS: &[&str] = &[
    EcPartner::uuid,
    EcPartner::user_id,
    EcPartner::is_checked_in,
    EcPartner::e_mail,
    EcPartner::role_id,
];
