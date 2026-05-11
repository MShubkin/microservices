use shared_essential::domain::{EcProtocol, EcProtocolItem};

pub(super) const UPDATE_PROTOCOL_DETAILS: &str =
    "/rest/estimated_commission/v1/get/protocol_update";

/// Поля позиций протокола, которые надо загрузить из базы.
pub(super) const ITEM_FIELDS_TO_LOAD: &[&str] = &[
    EcProtocolItem::uuid,
    EcProtocolItem::protocol_uuid,
    EcProtocolItem::source_uuid,
    EcProtocolItem::number,
    EcProtocolItem::is_excluded,
    EcProtocolItem::is_removed,
    EcProtocolItem::is_registered_by_d647,
    EcProtocolItem::result_id,
    EcProtocolItem::sum_excluded_vat,
    EcProtocolItem::pricing_sum_excluded_vat,
    EcProtocolItem::commission_sum_excluded_vat,
];

/// Поля протокола, которые могут измениться в результате запроса.
pub(super) const HEADER_FIELDS_TO_UPDATE: &[&str] = &[
    EcProtocol::protocol_type_id,
    EcProtocol::registration_number,
    EcProtocol::protocol_date,
    EcProtocol::pricing_organization_unit_id,
    EcProtocol::is_secret,
];

/// Поля позиции протокола, которые могут измениться в результате запроса.
pub(super) const ITEM_FIELDS_TO_UPDATE: &[&str] = &[
    // uuid позиции, ппз/дс и протокола не меняются
    // changed_.. выставляется в process_update
    // created_.. валиден при создании новой позиции, где пишутся все поля.
    EcProtocolItem::number,
    EcProtocolItem::is_registered_by_d647,
    EcProtocolItem::is_removed,
    EcProtocolItem::is_excluded,
    EcProtocolItem::sum_excluded_vat,
    EcProtocolItem::pricing_sum_excluded_vat,
    EcProtocolItem::commission_sum_excluded_vat,
    EcProtocolItem::result_id,
];
