use ahash::AHashSet;
use itertools::Itertools;
use shared_essential::{
    domain::PlanOrAmendment, presentation::dto::general::ObjectIdentifier,
};

use crate::common::ProcessingError;

/// Читает из БД ППЗ/ДС в соответствии с `select` и проверяет, что найдены все документы из набора `uuids`.
pub(super) fn check_plans_selection(
    plans: &[PlanOrAmendment],
    oids: &[ObjectIdentifier],
) -> Result<(), ProcessingError> {
    if plans.len() != oids.len() {
        let uuid_checker = plans.iter().map(|p| *p.uuid()).collect::<AHashSet<_>>();
        let ids = oids
            .iter()
            .filter(|i| !uuid_checker.contains(&i.uuid))
            .map(|i| i.id)
            .join(", ");
        return Err(ProcessingError::GetItemList(format!(
            "ППЗ/ДС с идентификаторами {ids} не были найдены для данного действия"
        )));
    }
    Ok(())
}
