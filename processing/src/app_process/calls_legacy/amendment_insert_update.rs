use crate::common::{ProcessingCtx, ProcessingError as PError, Result};

use shared_essential::domain::legacy::plans::PlanStatus;
use shared_essential::domain::processing::plan::PricingUnitId;
use shared_essential::domain::{
    ContractAmendmentItemRep, ContractAmendmentRep, DocumentApproverRep,
    PlanRetrospectiveRep,
};
use shared_essential::presentation::dto::processing::{
    AmendmentFromSrmParts, InsertUpdateSrmAmendmentsReq,
};
use shared_essential::presentation::dto::response_request::Messages;

use ahash::{AHashMap, AHashSet};

#[tracing::instrument(skip_all)]
pub(crate) async fn upsert_legacy_amendment(
    request: InsertUpdateSrmAmendmentsReq,
    proc_ctx: ProcessingCtx,
) -> Result<Messages> {
    trace_request(&request);

    let (headers, items, retros, spec_deps) =
        convert_to_local(request).map_err(|e| {
            tracing::error!(kind = "insert", "Ошибка в данных с монолита: {}", e);
            e
        })?;

    super::common::upsert(headers, items, retros, spec_deps, &proc_ctx)
        .await
        .map_err(|e| {
            tracing::error!(
                kind = "insert",
                "Ошибка при обновления с монолита: {}",
                e
            );
            e
        })
        .map(|(list, messages)| {
            tracing::info!(
                kind = "insert",
                "Успешно приняты ДС с монолита:\n{} заголовкoв.\n{} позиций",
                list.headers.len(),
                list.items.len()
            );
            messages
        })
}

fn trace_request(req: &InsertUpdateSrmAmendmentsReq) {
    if req.is_empty() {
        tracing::info!(
            kind = "insert",
            "Запрос от монолита на обновление ДС: {req:?}\n",
            req = req,
        );
    } else if let Some(item) = req.first() {
        tracing::info!(
            kind = "insert",
            "Запрос от монолита на обновление ДС:\n первый заголовок из {req_len}: {header:?}\n (Число позиций: {items_len} Число ретроспектив: {retro_len})",
            req_len = req.len(),
            header = item.header,
            items_len = item.items.len(),
            retro_len = item.retrospective_list.as_ref().map(|x| x.len()).unwrap_or(0),
        );
        // Print everything if we really need to.
        tracing::trace!(kind = "insert", "{:?}", req,);
    }
}

type LocalVariables<'a> = (
    Vec<ContractAmendmentRep>,
    Vec<ContractAmendmentItemRep>,
    Vec<PlanRetrospectiveRep>,
    Vec<DocumentApproverRep>,
);

/// Такой своеобразный перевод существует для того чтобы по УУИД проверить какие
/// существуют на нашей системе ППЗ чтобы знать что обновлять, а что вставлять.
fn convert_to_local<'a>(
    req: InsertUpdateSrmAmendmentsReq,
) -> Result<LocalVariables<'a>> {
    let mut headers = Vec::with_capacity(req.len());
    let mut item_map = AHashMap::new();
    let mut planning_version_checker = AHashSet::new();
    let mut retro_list = vec![];
    let mut spec_dep_list = vec![];

    for x in req {
        let AmendmentFromSrmParts {
            mut header,
            items,
            retrospective_list: retros,
            specialized_departments: spec_deps,
        } = x.try_to_part_rep()?;
        let id = header.id.unwrap_or_default();
        if !planning_version_checker.insert(id) {
            return Err(PError::SrmHeaderImport(id));
        }
        add_pricing_organization_id(&mut header);

        for mut item in items {
            // У позиций ППЗ не приходят header_uuid, они его берут из заголовка.
            item.header_uuid = header.uuid;
            item_map.insert(item.uuid, item);
        }
        retro_list.extend(retros);
        spec_dep_list.extend(spec_deps);
        headers.push(header);
    }
    let items = item_map.into_iter().map(|(_k, v)| v).collect();

    Ok((headers, items, retro_list, spec_dep_list))
}

/// ТОДО: Объединить в следующим обновлении
fn add_pricing_organization_id(p: &mut ContractAmendmentRep) {
    use PlanStatus::*;
    let Some(status) = p.status_id else {
        return;
    };
    let x = match status {
        ExecutorAppointmentD645
        | ExecutorAppointedD645
        | AnalysisPerformedD645
        | AnalysisCompletedD645 => PricingUnitId::D645,
        ExecutorAppointmentD646
        | ExecutorAppointedD646
        | AnalysisPerformedD646
        | AnalysisCompletedD646 => PricingUnitId::D646,
        ExecutorAppointmentD647
        | ExecutorAppointedD647
        | AnalysisPerformedD647
        | AnalysisCompletedD647 => PricingUnitId::D647,
        ExecutorAppointmentMTP
        | ExecutorAppointedMTP
        | AnalysisPerformedMTP
        | AnalysisCompletedMTP
        | LottingMTP => PricingUnitId::Gpk,
        _ => {
            return;
        }
    };
    p.pricing_organization_unit_id = Some(x);
}
