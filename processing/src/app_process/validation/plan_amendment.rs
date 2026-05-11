#![allow(unused)]

use ahash::AHashSet;

use shared_essential::domain::{
    EcAgendaItem, EcProtocolItem, PlanOrAmendment, PricingUnitId, ResultId,
};
use shared_essential::presentation::dto::response_request::{
    Message, MessageKind, Messages,
};

use super::{Extract, Validator};

pub struct PlanValidator<T> {
    entities: Vec<T>,
    messages: Messages,
    invalid_ids: AHashSet<i64>,
}

impl<T> PlanValidator<T> {
    pub fn new(entities: Vec<T>) -> Self {
        Self {
            entities,
            messages: Messages::default(),
            invalid_ids: AHashSet::default(),
        }
    }
}

impl<T> Validator<T> for PlanValidator<T>
where
    T: for<'ie> Extract<&'ie PlanOrAmendment>,
{
    fn is_valid(&self, entity: &T) -> bool {
        is_valid_plan(&self.invalid_ids, entity)
    }

    fn mark_invalid(&mut self, entity: &T, message: Message) {
        mark_invalid_plan(
            &mut self.invalid_ids,
            &mut self.messages,
            entity,
            message,
        );
    }

    fn has_errors(&self) -> bool {
        self.messages.is_error()
    }

    fn for_each<E, F, ErrFn>(&mut self, validate_fn: F, err_fn: ErrFn)
    where
        T: Extract<E>,
        F: Fn(E) -> bool,
        ErrFn: Fn(&T) -> Message,
    {
        for e in self.entities.iter() {
            let data: Option<E> = e.extract();
            if let Some(data) = data {
                if !validate_fn(data) {
                    mark_invalid_plan(
                        &mut self.invalid_ids,
                        &mut self.messages,
                        e,
                        err_fn(e),
                    );
                }
            }
        }
    }

    fn all<E, F>(&mut self, validate_fn: F, msg: Message)
    where
        T: Extract<E>,
        F: Fn(E) -> bool,
    {
        if self.entities.iter().filter_map(|e| e.extract()).all(validate_fn) {
            self.messages.add_prepared_message(msg)
        }
    }

    fn finalise(self) -> Vec<T> {
        self.entities
            .into_iter()
            .filter(|e| is_valid_plan(&self.invalid_ids, e))
            .collect()
    }
}

impl<T> PlanValidator<T>
where
    T: for<'e> Extract<&'e PlanOrAmendment>,
{
    /// Проверка, что все ППЗ/ДС имеют один и тот же [`PricingUnitId`]
    pub fn with_one_pricing_unit(
        &mut self,
        pricing_unit: PricingUnitId,
        msg: Message,
    ) {
        self.all(|p| *p.pricing_organization_unit_id() == pricing_unit, msg);
    }
}

/// Для проверок, когда ППЗ/ДС связан с элементом Повестки СК
pub struct PlanWithAgendaItem<'a> {
    pub plan: &'a PlanOrAmendment,
    pub agenda_item: &'a EcAgendaItem,
}

impl<T> PlanValidator<T>
where
    T: for<'e1> Extract<PlanWithAgendaItem<'e1>>
        + for<'e2> Extract<&'e2 PlanOrAmendment>,
{
    pub fn with_excluded_agenda_item<F>(&mut self, msg_fn: F)
    where
        F: Fn(&T) -> Message,
    {
        self.for_each(
            |extracted: PlanWithAgendaItem<'_>| extracted.agenda_item.is_excluded,
            msg_fn,
        );
    }
}

/// Для проверок, когда ППЗ/ДС связан с элементом Протокола СК
pub struct PlanWithProtocolItem<'a> {
    pub plan: &'a PlanOrAmendment,
    pub protocol_item: &'a EcProtocolItem,
}

impl<T> PlanValidator<T>
where
    T: for<'e1> Extract<PlanWithProtocolItem<'e1>>
        + for<'e2> Extract<&'e2 PlanOrAmendment>,
{
    pub fn with_protocol_item_result_id<F>(
        &mut self,
        result_id: ResultId,
        msg_fn: F,
    ) where
        F: Fn(&T) -> Message,
    {
        self.for_each(
            |extracted: PlanWithProtocolItem<'_>| {
                matches!(extracted.protocol_item.result_id, result_id)
            },
            msg_fn,
        );
    }
}

fn mark_invalid_plan<'a, T>(
    invalid_ids: &mut AHashSet<i64>,
    messages: &mut Messages,
    entity: &'a T,
    message: Message,
) where
    T: Extract<&'a PlanOrAmendment>,
{
    if let Some(plan) = entity.extract() {
        if message.kind >= MessageKind::Error {
            invalid_ids.insert(*plan.id());
        }
        messages.add_prepared_message(message);
    }
}

fn is_valid_plan<'a, T>(invalid_ids: &AHashSet<i64>, entity: &'a T) -> bool
where
    T: Extract<&'a PlanOrAmendment>,
{
    let plan = entity.extract();
    if let Some(plan) = plan {
        !invalid_ids.contains(plan.id())
    } else {
        false
    }
}
