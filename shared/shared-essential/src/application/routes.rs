use asez2_tables::{master_data::routes::RouteApprType, PlanOrAmendmentRep};

use crate::presentation::dto::master_data::request::{
    RouteFindReq, RouteFindReqItem, WithObjectTypeId,
};

/// Создает запрос к сервису NSI для получения маршрутов автоназначения
/// с заданным типом по данным ППЗ/ДС.
///
/// ```ignore
/// let plans = get_plans();
/// let req = find_auto_assignments_request(&plans, RouteApprType::PriceAnalysis);
/// let result = master_data_service.find_route::<_, AutoAssignExpertData>(req);
/// ```
pub fn create_find_auto_assignments_request(
    plans: &[PlanOrAmendmentRep],
    route_type: RouteApprType,
) -> RouteFindReq<WithObjectTypeId<&PlanOrAmendmentRep>> {
    RouteFindReq {
        type_id: route_type,
        item_list: plans
            .iter()
            .map(|plan| RouteFindReqItem {
                id: (*plan.id()).unwrap_or_default(),
                item: WithObjectTypeId::from(plan),
            })
            .collect(),
    }
}
