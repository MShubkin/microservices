use asez2_shared_db::db_item::AsezDate;
use shared_essential::{
    domain::maths::CurrencyValue, domain::*,
    presentation::dto::estimated_commission::UpdateProtocolReqWithUser,
};
use uuid::Uuid;

use super::UpdateProtocolError;

pub(crate) struct UpdateProtocolReqInner {
    pub user_id: i32,
    pub header: UpdateProtocolHeader,
    pub items: Vec<UpdateProtocolItem>,
    pub items_d647: Vec<UpdateProtocolItem>,
    pub partner_list: Vec<EcPartnerRep>,
    pub attachment_list: Vec<AttachmentRep>,
}

impl TryFrom<UpdateProtocolReqWithUser> for UpdateProtocolReqInner {
    type Error = UpdateProtocolError;

    fn try_from(
        value: UpdateProtocolReqWithUser,
    ) -> std::result::Result<Self, Self::Error> {
        let map_items = |items: Vec<EcProtocolItemRep>| {
            items
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<std::result::Result<_, _>>()
        };
        Ok(UpdateProtocolReqInner {
            user_id: value.user_id,
            header: value.header.try_into()?,
            items: map_items(value.items)?,
            items_d647: map_items(value.items_d647)?,
            partner_list: value.partner_list,
            attachment_list: value.attachment_list,
        })
    }
}

impl From<UpdateProtocolReqInner> for UpdateProtocolReqWithUser {
    fn from(
        UpdateProtocolReqInner {
            user_id,
            header,
            items,
            items_d647,
            partner_list,
            attachment_list,
        }: UpdateProtocolReqInner,
    ) -> Self {
        UpdateProtocolReqWithUser {
            user_id,
            header: header.into(),
            items: items.into_iter().map(From::from).collect(),
            items_d647: items_d647.into_iter().map(From::from).collect(),
            partner_list,
            attachment_list,
        }
    }
}

#[derive(Debug)]
pub(crate) struct UpdateProtocolHeader {
    pub id: i64,
    pub uuid: Uuid,
    pub protocol_type_id: ProtocolType,
    pub registration_number: Option<String>,
    pub protocol_date: AsezDate,
    pub pricing_organization_unit_id: Option<PricingUnitId>,
    pub is_secret: bool,
}

macro_rules! unwrap_or_missing {
    ($val:expr, $field:ident) => {
        $val.$field
            .ok_or(UpdateProtocolError::MissingField(stringify!($field)))?
    };
}

impl TryFrom<EcProtocolRep> for UpdateProtocolHeader {
    type Error = UpdateProtocolError;

    fn try_from(value: EcProtocolRep) -> std::result::Result<Self, Self::Error> {
        Ok(UpdateProtocolHeader {
            id: unwrap_or_missing!(value, id),
            uuid: unwrap_or_missing!(value, uuid),
            protocol_type_id: unwrap_or_missing!(value, protocol_type_id),
            registration_number: value.registration_number.flatten(),
            protocol_date: value
                .protocol_date
                .ok_or(UpdateProtocolError::MissingProtocolDate)?,
            pricing_organization_unit_id: value.pricing_organization_unit_id,
            is_secret: unwrap_or_missing!(value, is_secret),
        })
    }
}

impl From<UpdateProtocolHeader> for EcProtocol {
    fn from(value: UpdateProtocolHeader) -> Self {
        let UpdateProtocolHeader {
            id,
            uuid,
            protocol_type_id,
            registration_number,
            protocol_date,
            pricing_organization_unit_id,
            is_secret,
        } = value;
        EcProtocol {
            uuid,
            id,
            protocol_type_id,
            registration_number,
            pricing_organization_unit_id: pricing_organization_unit_id
                .unwrap_or_default(),
            is_secret,
            protocol_date,
            ..Default::default()
        }
    }
}

impl From<UpdateProtocolHeader> for EcProtocolRep {
    fn from(
        UpdateProtocolHeader {
            id,
            uuid,
            protocol_type_id,
            registration_number,
            protocol_date,
            pricing_organization_unit_id,
            is_secret,
        }: UpdateProtocolHeader,
    ) -> Self {
        EcProtocolRep {
            id: Some(id),
            uuid: Some(uuid),
            protocol_type_id: Some(protocol_type_id),
            registration_number: Some(registration_number),
            protocol_date: Some(protocol_date),
            pricing_organization_unit_id,
            is_secret: Some(is_secret),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub(crate) struct UpdateProtocolItem {
    pub uuid: Option<Uuid>,
    pub source_uuid: Uuid,
    pub is_removed: bool,
    pub is_excluded: bool,
    pub sum_excluded_vat: Option<CurrencyValue>,
    pub pricing_sum_excluded_vat: Option<CurrencyValue>,
    pub commission_sum_excluded_vat: Option<CurrencyValue>,
    pub result_id: Option<ResultId>,
}

impl TryFrom<EcProtocolItemRep> for UpdateProtocolItem {
    type Error = UpdateProtocolError;

    fn try_from(
        value: EcProtocolItemRep,
    ) -> std::result::Result<Self, Self::Error> {
        // Если пользователь создает новый элемент, то он может не передать
        // суммовые поля, которые нам надо вручную заполнить
        // По старым элементам суммовые поля должны быть переданы
        let (
            sum_excluded_vat,
            pricing_sum_excluded_vat,
            commission_sum_excluded_vat,
        ) = if value.uuid.is_some() {
            (
                unwrap_or_missing!(value, sum_excluded_vat),
                unwrap_or_missing!(value, pricing_sum_excluded_vat),
                unwrap_or_missing!(value, commission_sum_excluded_vat),
            )
        } else {
            (
                value.sum_excluded_vat.flatten(),
                value.pricing_sum_excluded_vat.flatten(),
                value.commission_sum_excluded_vat.flatten(),
            )
        };

        Ok(UpdateProtocolItem {
            uuid: value.uuid,
            source_uuid: unwrap_or_missing!(value, source_uuid),
            is_removed: value.is_removed.unwrap_or(false),
            is_excluded: unwrap_or_missing!(value, is_excluded),
            sum_excluded_vat,
            pricing_sum_excluded_vat,
            commission_sum_excluded_vat,
            result_id: value.result_id,
        })
    }
}

impl From<UpdateProtocolItem> for EcProtocolItemRep {
    fn from(
        UpdateProtocolItem {
            uuid,
            source_uuid,
            is_removed,
            is_excluded,
            sum_excluded_vat,
            pricing_sum_excluded_vat,
            commission_sum_excluded_vat,
            result_id,
        }: UpdateProtocolItem,
    ) -> Self {
        EcProtocolItemRep {
            uuid,
            source_uuid: Some(source_uuid),
            is_removed: is_removed.then_some(true),
            is_excluded: Some(is_excluded),
            sum_excluded_vat: Some(sum_excluded_vat),
            pricing_sum_excluded_vat: Some(pricing_sum_excluded_vat),
            commission_sum_excluded_vat: Some(commission_sum_excluded_vat),
            result_id,
            ..Default::default()
        }
    }
}
