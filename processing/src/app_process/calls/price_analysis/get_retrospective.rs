use std::sync::Arc;

use asez2_shared_db::db_item::joined::JoinTo;
use asez2_shared_db::db_item::{from_item_with_fields, Select, SelectionKind};

use shared_essential::presentation::dto::processing::{
    GetRetrospectiveReq, GetRetrospectiveResponseData, MergedPlanRetrospective,
};

use shared_essential::{
    domain::{
        ContractAmendment, ContractAmendmentRep,
        JoinedPlanRetrospectivePlanContractAmendmentStatusHistorySelector, Plan,
        PlanOrAmendmentRep, PlanRep, PlanRetrospective, StatusHistory,
        StatusHistoryRep,
    },
    presentation::dto::response_request::ApiResponse,
};
use sqlx::PgPool;

pub(crate) async fn get_retrospective(
    dto: GetRetrospectiveReq,
    db_pool: Arc<PgPool>,
) -> crate::common::Result<ApiResponse<GetRetrospectiveResponseData, ()>> {
    tracing::trace!(
        kind = "get",
        "Получен запрос на выборку ретроспективных ппз/дс: {req:?}\n",
        req = dto,
    );
    let mut response = ApiResponse::default();

    let retrospective_fields =
        &[PlanRetrospective::plan_year, PlanRetrospective::id];
    let retrospective_select = Select::with_fields(retrospective_fields)
        .eq(PlanRetrospective::is_removed, false)
        .add_expand_filter(
            PlanRetrospective::plan_uuid,
            SelectionKind::In,
            dto.item_list.iter().map(|i| i.uuid),
        );
    let plan_fields = &[
        Plan::contract_subject,
        Plan::pricing_expert_id,
        Plan::pricing_resume,
        Plan::pricing_sum_excluded_vat,
    ];
    let mut plan_rep_fields = plan_fields.to_vec();
    plan_rep_fields.extend_from_slice(&[
        "plan_id",
        "contract_subject_short",
        "pricing_resume_short",
    ]);

    let status_history_fields = &[StatusHistory::created_at];
    let status_history_select = Select::with_fields(status_history_fields)
        .add_expand_filter(
            StatusHistory::status_id,
            SelectionKind::In,
            [225, 345, 355],
        );
    let plan_select = Select::with_fields(plan_fields);
    let joined_select =
        JoinedPlanRetrospectivePlanContractAmendmentStatusHistorySelector::new(
            retrospective_select,
        )
        .set_plan(Plan::join_default().selecting(plan_select.clone()))
        .set_amendment(
            ContractAmendment::join_default().selecting(plan_select.clone()),
        )
        .set_status_history(
            StatusHistory::join_default().selecting(status_history_select.clone()),
        );

    let results = joined_select.get(db_pool.as_ref()).await?;

    let mut item_list: Vec<MergedPlanRetrospective> =
        Vec::with_capacity(results.len());

    let from_retrospective = from_item_with_fields(retrospective_fields);
    let from_status_history = from_item_with_fields(status_history_fields);
    let from_plan = from_item_with_fields(&plan_rep_fields);
    let from_ca = from_item_with_fields(&plan_rep_fields);
    for result in results {
        let plan_retrospective_rep = from_retrospective(result.plan_retrospective);

        let status_history_rep = result
            .status_history
            .iter()
            .max_by_key(|item| item.created_at)
            .cloned()
            .map_or_else(StatusHistoryRep::default, &from_status_history);

        if let Some(plan) = result.plan {
            let plan: PlanRep = from_plan(plan);
            item_list.push(MergedPlanRetrospective {
                plan: PlanOrAmendmentRep::from(plan),
                retrospective: plan_retrospective_rep,
                status_history: status_history_rep,
            });
        } else if let Some(amendment) = result.amendment {
            let amendment: ContractAmendmentRep = from_ca(amendment);
            item_list.push(MergedPlanRetrospective {
                plan: PlanOrAmendmentRep::from(amendment),
                retrospective: plan_retrospective_rep,
                status_history: status_history_rep,
            });
        }
    }
    response.data = GetRetrospectiveResponseData { item_list };
    Ok(response)
}
