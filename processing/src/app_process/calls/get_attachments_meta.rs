use std::sync::Arc;

use ahash::AHashSet;
use asez2_shared_db::db_item::{joined::JoinTo, Select};
use itertools::Itertools;
use shared_essential::{
    domain::{
        Attachment, ContractAmendment, ContractAmendmentWithAttachmentsSelector,
        Plan, PlanWithAttachmentsSelector,
    },
    presentation::dto::{
        general::ObjectIdentifier,
        processing::{
            AttachmentMeta, GetAttachmentsMetaRequest,
            GetAttachmentsMetaResponseData, GetAttachmentsMetaResponseItem,
        },
        response_request::{ApiResponse, EntityKind},
    },
};
use sqlx::PgPool;

use crate::common::{ProcessingError, Result};

pub(crate) async fn get_attachments_meta(
    req: GetAttachmentsMetaRequest,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetAttachmentsMetaResponseData, ()>> {
    let db_pool = db_pool.as_ref();
    let GetAttachmentsMetaRequest { item_list } = req;

    if item_list.is_empty() {
        return Ok(ApiResponse::default());
    }

    let plan_uuids = item_list.iter().map(|i| i.uuid);

    let attachment_select = Select::with_fields([
        Attachment::uuid,
        Attachment::category_id,
        Attachment::parent_number,
    ])
    .eq(Attachment::is_removed, false);

    let plans = PlanWithAttachmentsSelector::new(
        Select::with_fields([Plan::uuid, Plan::id])
            .in_any(Plan::uuid, plan_uuids.clone()),
    )
    .set_attachments(
        Attachment::join_default().selecting(attachment_select.clone()),
    )
    .get(db_pool)
    .await?;
    let amendments = ContractAmendmentWithAttachmentsSelector::new(
        Select::with_fields([ContractAmendment::uuid, ContractAmendment::id])
            .in_any(ContractAmendment::uuid, plan_uuids),
    )
    .set_attachments(Attachment::join_default().selecting(attachment_select))
    .get(db_pool)
    .await?;

    if plans.len() + amendments.len() != item_list.len() {
        let found_ids = plans
            .iter()
            .map(|x| x.plan.id)
            .chain(amendments.iter().map(|x| x.amendment.id))
            .collect::<AHashSet<_>>();

        let missing = item_list
            .iter()
            .filter(|id| !found_ids.contains(&id.id))
            .map(|id| id.id.to_string())
            .join(", ");

        let msg =
            format!("Записи ППЗ/ДС c идентификаторами {} не найдены", missing);
        return Err(ProcessingError::GetItemList(msg));
    }

    let merged_plans = plans
        .into_iter()
        .map(|i| (i.plan.id, i.plan.uuid, i.attachments, EntityKind::Plan))
        .chain(amendments.into_iter().map(|i| {
            (
                i.amendment.id,
                i.amendment.uuid,
                i.attachments,
                EntityKind::ContractAmendment,
            )
        }))
        .map(|(id, uuid, attachment_list, object_type)| {
            let id = ObjectIdentifier::new_with_type(id, uuid, object_type);
            let attachment_list = attachment_list
                .into_iter()
                .map(|i| AttachmentMeta {
                    uuid: i.uuid,
                    category_id: i.category_id,
                    parent_number: i.parent_number.unwrap_or_default(),
                })
                .collect();

            GetAttachmentsMetaResponseItem {
                id,
                attachment_list,
            }
        })
        .collect();

    Ok(ApiResponse::default().with_data(merged_plans))
}
