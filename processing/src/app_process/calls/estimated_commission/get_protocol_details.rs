use std::collections::HashMap;
use std::{cmp::Ordering, sync::Arc};

use asez2_shared_db::db_item::joined::JoinTo;
use asez2_shared_db::db_item::{AdaptorableIter, Select};
use asez2_shared_db::DbAdaptor;
use itertools::Itertools;
use shared_essential::{
    domain::*,
    presentation::dto::{
        processing::{
            Calculated, GetProtocolDetailsReq, GetProtocolDetailsRes,
            ProtocolDetailsItem,
        },
        response_request::*,
    },
};

use crate::common::{ProcessingError as PError, Result};

use sqlx::PgPool;
use uuid::Uuid;

const GET_PROTOCOL_DETAILS: &str =
    "/rest/estimated_commission/v1/get/protocol_details";

const PROTOCOL_FIELDS: &[&str] = &[
    EcProtocol::uuid,
    EcProtocol::id,
    EcProtocol::protocol_type_id,
    EcProtocol::registration_number,
    EcProtocol::status_id,
    EcProtocol::pricing_organization_unit_id,
    EcProtocol::is_removed,
    EcProtocol::is_secret,
    EcProtocol::protocol_date,
    EcProtocol::created_at,
    EcProtocol::changed_at,
    EcProtocol::created_by,
    EcProtocol::changed_by,
];

const PROTOCOL_ITEM_FIELDS: &[&str] = &[
    EcProtocolItem::uuid,
    EcProtocolItem::sum_excluded_vat,
    EcProtocolItem::source_uuid,
    EcProtocolItem::result_id,
    EcProtocolItem::pricing_sum_excluded_vat,
    EcProtocolItem::number,
    EcProtocolItem::is_registered_by_d647,
    EcProtocolItem::is_excluded,
    EcProtocolItem::commission_sum_excluded_vat,
];

/// (Если protocol_type_id = 2, передаем значение данного поля из ППЗ/ДС)
const PLAN_FIELDS_2: &[&str] = &[
    Plan::contract_subject,
    Plan::currency_id,
    Plan::customer_id,
    Plan::number_customer,
    "plan_id",
    Plan::pricing_expert_id,
    Plan::pricing_resume,
    Plan::section_id,
    Plan::status_id,
    Plan::supplier_id,
];

const PARTNER_FIELDS: &[&str] = &[
    EcPartner::uuid,
    EcPartner::user_id,
    EcPartner::is_checked_in,
    EcPartner::e_mail,
    EcPartner::role_id,
];

const ATTACHMENT_FIELDS: &[&str] = &[
    Attachment::uuid,
    Attachment::number,
    Attachment::name,
    Attachment::category_id,
    Attachment::changed_at,
    Attachment::changed_by,
    Attachment::created_at,
    Attachment::created_by,
    Attachment::is_classified,
    Attachment::is_removed,
    Attachment::kind_id,
    Attachment::mime_id,
    Attachment::parent_number,
    Attachment::mime_id,
    Attachment::size,
];

#[tracing::instrument(skip_all)]
pub(crate) async fn get_protocol_details(
    request: GetProtocolDetailsReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetProtocolDetailsRes, ()>> {
    tracing::info!(
        kind = "get",
        "Процессинг: Получение подробностей протокола СК ({get}): {req:?}\n",
        req = request,
        get = GET_PROTOCOL_DETAILS
    );

    let GetProtocolDetailsReq { id } = request;

    let protocol = get_protocol(id, &db_pool).await?;

    let reply = convert(protocol)?;

    Ok((reply, Messages::default()).into())
}

pub(crate) async fn get_protocol(
    id: i64,
    pool: &PgPool,
) -> Result<ProtocolDetails> {
    let protocol_select =
        Select::full::<EcProtocol>().eq(EcProtocol::id, id).take_first();

    let protocol_item_sel =
        Select::full::<EcProtocolItem>().eq(EcProtocolItem::is_removed, false);
    let partner_select =
        Select::full::<EcPartner>().eq(EcPartner::is_removed, false);
    let attachment_select =
        Select::full::<Attachment>().eq(Attachment::is_removed, false);

    let mut joined_protocols = ProtocolDetailsSelector::new(protocol_select)
        .set_items(
            EcProtocolItem::join_default().selecting(protocol_item_sel).distinct(),
        )
        .set_partner_list(
            EcPartner::join_default().selecting(partner_select).distinct(),
        )
        .set_plans(Plan::join_default().distinct())
        .set_amendments(ContractAmendment::join_default().distinct())
        .set_attachment_list(
            Attachment::join_default().selecting(attachment_select).distinct(),
        )
        .distinct()
        .get(pool)
        .await?;

    joined_protocols.pop().ok_or_else(|| {
        let msg = format!("Протокол № {} не найден.", id);
        PError::GetProtocolDetails(msg)
    })
}

/// TODO: как то DISTINCT своё дело не делает.
fn convert(protocol: ProtocolDetails) -> Result<GetProtocolDetailsRes> {
    let ProtocolDetails {
        protocol,
        items,
        plans,
        amendments,
        partner_list,
        attachment_list,
    } = protocol;

    let mut partner_list = partner_list
        .into_iter()
        .unique_by(|p| p.uuid) // Just in case.
        .adaptors_with_fields(PARTNER_FIELDS)
        .collect::<Vec<EcPartnerRep>>();

    let attachment_list = attachment_list
        .into_iter()
        .unique_by(|a| a.uuid)
        .adaptors_with_fields(ATTACHMENT_FIELDS)
        .collect::<Vec<AttachmentRep>>();

    let protocol =
        EcProtocolRep::from_item::<&str>(protocol, Some(PROTOCOL_FIELDS));

    let plan_checker = PlanOrAmendment::collect_map_by_uuid(plans, amendments);
    let (mut protocol_item_list, mut protocol_item_d647_list) =
        convert_protocol_item_related(items, plan_checker)?;

    protocol_item_list.sort_by(|a, b| {
        a.protocol_item.item.number.cmp(&b.protocol_item.item.number)
    });
    protocol_item_d647_list.sort_by(|a, b| {
        a.protocol_item.item.number.cmp(&b.protocol_item.item.number)
    });
    // Сортируем партнёров по роли и пользователю.
    partner_list.sort_unstable_by(|a, b| {
        match a.commission_role_id.cmp(&b.commission_role_id) {
            Ordering::Equal => a.user_id.cmp(&b.user_id),
            x => x,
        }
    });

    Ok(GetProtocolDetailsRes {
        protocol,
        protocol_item_list,
        protocol_item_d647_list,
        partner_list,
        attachment_list,
    })
}

/// Create the related structure
fn convert_protocol_item_related(
    protocol_items: Vec<EcProtocolItem>,
    plan_checker: HashMap<Uuid, PlanOrAmendment>,
) -> Result<(Vec<ProtocolDetailsItem>, Vec<ProtocolDetailsItem>)> {
    let (mut item_list, mut item_d647_list) = (Vec::new(), Vec::new());

    for item in protocol_items.into_iter().unique_by(|a| a.uuid) {
        let p = plan_checker.get(&item.source_uuid).cloned().ok_or_else(|| {
            PError::GetProtocolDetails(format!(
                "Отсутствует ППЗ/ДС по позиции протокола № {}",
                item.number
            ))
        })?;
        let actual_item = convert_inner(p, item)?;
        if actual_item
            .protocol_item
            .item
            .is_registered_by_d647
            .expect("Должно быть, так как было запрошено")
        {
            item_d647_list.push(actual_item);
        } else {
            item_list.push(actual_item);
        }
    }

    Ok((item_list, item_d647_list))
}

pub(crate) fn convert_inner(
    plan: PlanOrAmendment,
    protocol_item: EcProtocolItem,
) -> Result<ProtocolDetailsItem> {
    let sum_excluded_vat = protocol_item.sum_excluded_vat.unwrap_or_default();
    let pricing_sum_excluded_vat = match plan {
        PlanOrAmendment::Plan(ref p) => p.pricing_sum_excluded_vat,
        PlanOrAmendment::Amendment(ref a) => {
            a.pricing_delta_sum_excluded_vat.unwrap_or_default()
        }
    };
    let commission_sum_excluded_vat =
        protocol_item.commission_sum_excluded_vat.unwrap_or_default();

    let commission_economy_sum = sum_excluded_vat - commission_sum_excluded_vat;

    let percent_economy = if commission_sum_excluded_vat.is_negative()
        || !sum_excluded_vat.is_positive()
    {
        "-".to_string()
    } else {
        const C: f64 = 100.; // Roman numeral
        let economy: f64 = commission_economy_sum.into();
        let sum: f64 = sum_excluded_vat.into();
        let x = economy / sum * C.powi(2);
        let x = x.round() / C;
        // Локализация, дорогая ты моя, не место тебе здесь в нашем
        // убогом бэке.
        // Подняться тебе бы до светлого, блистающего фронта, но увы,
        // не суждено....
        format!("{x:.2}").replace('.', ",")
    };

    // Задействован механизм расчёта расчётных полей.
    let protocol_item =
        Calculated::from_db_item(protocol_item, Some(PROTOCOL_ITEM_FIELDS))
            .set_commission_economy_sum_excluded_vat_unconditional(
                commission_economy_sum,
            )
            .set_commission_percent_economy_unconditional(percent_economy)
            .set_is_commission_sum_equal_actual_sum_unconditional(
                pricing_sum_excluded_vat == commission_sum_excluded_vat,
            )
            .set_actual_sum_excluded_vat_unconditional(pricing_sum_excluded_vat);

    let plan = PlanOrAmendmentRep::from_item(plan, Some(PLAN_FIELDS_2));

    Ok(ProtocolDetailsItem {
        protocol_item,
        plan,
    })
}
