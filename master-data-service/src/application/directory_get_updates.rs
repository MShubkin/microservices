use crate::application::master_data::{get_master_data, MasterDataCommonDirectory};
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::DbItem;
use futures::future::{try_join_all, BoxFuture};
use shared_essential::presentation::dto::master_data::error::MasterDataResult;
use shared_essential::presentation::dto::master_data::updates::{
    MasterDataUpdate, MasterDataUpdateEntity, MasterDataUpdates,
};

use crate::domain::{MasterDataDirectoryInterface, MasterDataRecord};

async fn handle_update<A, FN>(
    directory: &MasterDataCommonDirectory<A>,
    changed_at: AsezTimestamp,
    make_entity: FN,
) -> MasterDataResult<Option<MasterDataUpdate>>
where
    A: MasterDataRecord + Sync + Send + DbItem,
    FN: FnOnce(AsezTimestamp, Vec<A>) -> MasterDataUpdate,
{
    let updates = directory.get_updates(changed_at).await?;
    if updates.is_empty() {
        Ok(None)
    } else {
        let entity = make_entity(directory.changed_at, updates);
        Ok(Some(entity))
    }
}

async fn handle_full_dictionary<A, Fn>(
    directory: &MasterDataCommonDirectory<A>,
    make_fn: Fn,
) -> MasterDataResult<Option<MasterDataUpdate>>
where
    A: MasterDataRecord + Sync + Send + DbItem,
    Fn: FnOnce(Vec<A>) -> MasterDataUpdateEntity,
{
    let dicts = directory.get_full_data().await?;
    if dicts.is_empty() {
        Ok(None)
    } else {
        let record = make_fn(dicts);
        Ok(Some(MasterDataUpdate {
            changed_at: AsezTimestamp::from_unix_timestamp(0),
            entity: record,
        }))
    }
}

#[allow(clippy::vec_init_then_push)]
pub(crate) async fn directory_get_updates(
    timestamp: i64,
) -> MasterDataResult<MasterDataUpdates> {
    let master_data = get_master_data()?;
    let changed_at = AsezTimestamp::from_unix_timestamp(timestamp);

    let mut update_results: Vec<
        BoxFuture<'static, MasterDataResult<Option<MasterDataUpdate>>>,
    > = Vec::new();

    // Некоторые справочники должны возвращаться в любом случае,
    // независимо от таймстемпа их обновления
    update_results.push(Box::pin(handle_full_dictionary(
        &master_data.favorite_dictionaries,
        MasterDataUpdateEntity::FavoriteDictionary,
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.agenda_status,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::AgendaStatus(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.estimated_commission_result,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::EstimatedCommissionResult(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.estimated_commission_role,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::EstimatedCommissionRole(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.pricing_organization_unit,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::PricingOrganizationUnit(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.estimated_commission_protocol_status,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::ProtocolStatus(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.protocol_type,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::ProtocolType(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.critical_type_color_scheme,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::CriticalTypeColorScheme(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.expert_conclusion_type,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::ExpertConclusionType(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.object_type,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::ObjectType(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.output_form,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::OutputForm(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.response,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::DepartmentResponseStatus(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.plan_reasons_cancel_check_reason,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::PlanReasonCancelCheckReason(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.plan_reasons_cancel_functionality,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::PlanReasonCancelFunctionality(values),
        },
    )));

    update_results.push(Box::pin(handle_update(
        &master_data.plan_reasons_cancel_impact_area,
        changed_at,
        |changed_at, values| MasterDataUpdate {
            changed_at,
            entity: MasterDataUpdateEntity::PlanReasonCancelImpactArea(values),
        },
    )));

    let mut master_data_updates = MasterDataUpdates::default();

    // Get results
    for result in try_join_all(update_results).await?.iter().flatten() {
        if result.changed_at > master_data_updates.changed_at {
            master_data_updates.changed_at = result.changed_at;
        }
        master_data_updates.entity_list.push(result.clone());
    }

    Ok(master_data_updates)
}
