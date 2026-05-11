use crate::application::master_data::MasterData;
use asez2_shared_db::DbItem;
use shared_essential::{
    domain::enums::master_data::DirectoryType,
    presentation::dto::{
        master_data::{
            error::MasterDataResult,
            request::SearchByUserInput,
            response::{
                DirectoryRecord, DirectoryRecordResponse, DirectoryRecords,
                MasterDataSearchResponse,
            },
        },
        response_request::Messages,
    },
};

use crate::domain::{MasterDataDirectoryInterface, MasterDataRecord};

pub mod action;
pub mod directory_get_updates;
pub mod hierarchical_values;
pub mod master_data;
pub mod routes;

async fn process_search_by_id<A, M, F>(
    master_data: &M,
    ids: &[i32],
    new_record_with_data_fn: F,
) -> MasterDataResult<(Messages, DirectoryRecordResponse)>
where
    A: MasterDataRecord + DbItem,
    M: MasterDataDirectoryInterface<A>,
    F: FnOnce(Vec<A>) -> DirectoryRecord,
{
    let (messages, result) = master_data.get_by_ids(ids).await?;

    Ok((
        messages,
        DirectoryRecordResponse {
            value: new_record_with_data_fn(result),
        },
    ))
}

async fn process_search<A, M, F>(
    master_data: &M,
    search_request: &SearchByUserInput,
    new_record_with_data_fn: F,
) -> MasterDataResult<(Messages, DirectoryRecordResponse)>
where
    A: MasterDataRecord + DbItem,
    M: MasterDataDirectoryInterface<A>,
    F: FnOnce(Vec<A>) -> DirectoryRecord,
{
    let (messages, result) = master_data.search(search_request).await?;

    Ok((
        messages,
        DirectoryRecordResponse {
            value: new_record_with_data_fn(result),
        },
    ))
}

async fn process_get_full_data<'a, A, M, F>(
    master_data: &M,
    _: (),
    new_record_with_data_fn: F,
) -> MasterDataResult<DirectoryRecord>
where
    A: MasterDataRecord + DbItem,
    M: MasterDataDirectoryInterface<A>,
    F: FnOnce(Vec<A>) -> DirectoryRecord,
{
    let result = master_data.get_full_data().await?;
    Ok(new_record_with_data_fn(result))
}

macro_rules! gen_match_by_directory {
    (
        $func:tt, $match_type:expr, $ids:expr, {
            $(
                $variant:ident => $field:expr
            ),* $(,)?
        },
    ) => {
        match $match_type {
            $(
                DirectoryType::$variant => {
                    $func(&$field, $ids, DirectoryRecord::$variant).await
                }
            ),*
        }
    };
}

macro_rules! process_by_directory {
    ($func:tt, $match_type:expr, $ids:expr, $master_data:tt) => {
        gen_match_by_directory!(
            $func, $match_type, $ids,
            {
                AnalysisMethod => $master_data.analysis_method,
                AssigningExecutorMethod => $master_data.assigning_executor_method,
                BudgetItem => $master_data.budget_item,
                Category => $master_data.category,
                CriticalTypeColorScheme => $master_data.critical_type_color_scheme,
                EstimatedCommissionAgendaStatus => $master_data.agenda_status,
                EstimatedCommissionProtocolStatus => $master_data.estimated_commission_protocol_status,
                EstimatedCommissionProtocolType => $master_data.protocol_type,
                EstimatedCommissionResult => $master_data.estimated_commission_result,
                ExpertConclusionType => $master_data.expert_conclusion_type,
                ObjectType => $master_data.object_type,
                Organization => $master_data.organization,
                PpzType => $master_data.ppz_type,
                PriceAnalysisMethod => $master_data.price_analysis_method,
                PricingUnit => $master_data.pricing_organization_unit,
                SchedulerRequestUpdateCatalog => $master_data.scheduler_request_update_catalog,
                PriceInformationRequestType => $master_data.price_information_request_type,
                TcpStatus => $master_data.technical_commercial_proposal_status,
                Okpd2 => $master_data.okpd2,
                OrganizationalStructure => $master_data.organizational_structure,
                PaymentConditions => $master_data.payment_conditions,
                DepartmentResponseStatus => $master_data.response,
                PlanReasonCancelImpactArea => $master_data.plan_reasons_cancel_impact_area,
                PlanReasonCancelFunctionality => $master_data.plan_reasons_cancel_functionality,
                PlanReasonCancelCheckReason => $master_data.plan_reasons_cancel_check_reason
            },
        )
    };
}

pub(crate) async fn directory_search_by_id(
    master_data: &MasterData,
    directory_type: DirectoryType,
    ids: &[i32],
) -> MasterDataResult<(Messages, DirectoryRecordResponse)> {
    process_by_directory!(process_search_by_id, directory_type, ids, master_data)
}

pub(crate) async fn directory_search_by_user_input(
    master_data: &MasterData,
    directory_type: DirectoryType,
    search_request: &SearchByUserInput,
) -> MasterDataResult<(Messages, DirectoryRecordResponse)> {
    process_by_directory!(
        process_search,
        directory_type,
        search_request,
        master_data
    )
}

pub(crate) async fn directory_get_full_data(
    master_data: &MasterData,
    directory_types: Vec<DirectoryType>,
) -> MasterDataResult<MasterDataSearchResponse> {
    let mut records = Vec::with_capacity(directory_types.len());
    for directory_type in directory_types {
        let record = process_by_directory!(
            process_get_full_data,
            directory_type,
            (),
            master_data
        )?;
        records.push(record);
    }

    Ok(MasterDataSearchResponse {
        messages: Messages::default(),
        records: DirectoryRecords::from_iter(records),
    })
}
