//! This module is responsible for insert accesses and updates where a record needs
//! to be kept of what was changed, how and when.
use std::sync::Arc;

use crate::common::{rules::*, ProcessingError, Result};

use ahash::AHashMap;
use asez2_shared_db::db_item::{update_fields_helper, DbAdaptor};
use shared_essential::{
    application::records::{Recorder, RulesLawyer, StatusHandler},
    domain::{
        ContractAmendment, ContractAmendmentRep, Plan, PlanOrAmendment,
        PlanOrAmendmentRep, PlanRep, Section,
    },
    presentation::dto::{
        general::{ObjectIdentifier, ObjectIdentifierWithStatusNote},
        processing::{PreCancelPlansReq, PreChangeFormReq},
        response_request::{Message, MessageKind, Messages},
    },
};
use sqlx::PgPool;

mod send_to_monolith;

pub(crate) use self::send_to_monolith::{send_plans_to_monolith, send_to_monolith};

#[derive(Debug, Clone)]
pub(crate) struct ProcessingRulesChecker {
    rules: Arc<ProcessingRules>,
    db_pool: Arc<PgPool>,
}

#[async_trait::async_trait]
impl StatusHandler for ProcessingRulesChecker {
    type Error = ProcessingError;

    async fn check_insert<T: RulesLawyer>(
        &self,
        _new: &[T],
        _messages: &mut Messages,
    ) -> std::result::Result<bool, Self::Error> {
        Ok(true)
    }

    async fn check_update<T: RulesLawyer>(
        &self,
        fields_to_update: &[&str],
        new: &[T],
        old: &[T],
        messages: &mut Messages,
    ) -> std::result::Result<bool, Self::Error> {
        Ok(Self::check_lengths(new, old, messages)
            && (!fields_to_update.contains(&"status_id")
                || self.check_plan_status(new, old, messages).await?))
    }

    async fn check_upsert<T: RulesLawyer>(
        &self,
        fields_to_update: &[&str],
        new: &[T],
        old: &[T],
        messages: &mut Messages,
    ) -> std::result::Result<bool, Self::Error> {
        Ok(!fields_to_update.contains(&"status_id")
            || self.check_plan_status(new, old, messages).await?)
    }
}

impl ProcessingRulesChecker {
    pub(crate) fn new(rules: Arc<ProcessingRules>, db_pool: Arc<PgPool>) -> Self {
        ProcessingRulesChecker { rules, db_pool }
    }

    fn check_lengths<R: RulesLawyer>(
        subjects: &[R],
        old: &[R],
        messages: &mut Messages,
    ) -> bool {
        if subjects.len() != old.len() {
            messages.add_prepared_message(Message::stop(
                "Количество записей ППЗ/ДС в БД не консистентны".to_string(),
            ));
            false
        } else {
            true
        }
    }

    async fn check_plan_status<R: RulesLawyer>(
        &self,
        subjects: &[R],
        old: &[R],
        messages: &mut Messages,
    ) -> Result<bool> {
        use crate::app_process::{pre_cancel_plan, pre_change_form};

        let filtered_plans = subjects
            .iter()
            .zip(old.iter())
            .filter(|(a, b)| a.plan_status() != b.plan_status());

        let status_rules = &self.rules.status_rules();

        let mut rules_map = AHashMap::new();

        for (new, old) in filtered_plans {
            if let Some(rule) =
                status_rules.get_rule(old.plan_status(), new.plan_status())
            {
                let e = rules_map.entry(rule).or_insert_with(Vec::new);
                e.push(ObjectIdentifierWithStatusNote {
                    inner: ObjectIdentifier::new(new.id(), new.uuid()),
                    ..Default::default()
                });
            } else {
                let msg = format!(
                    "Переход статуса с \"{}\" на \"{}\" не разрешен (ППЗ/ДС номер {})",
                    old.plan_status(),
                    new.plan_status(),
                    old.id()
                );
                messages.add_message(MessageKind::Error, msg);
            }
        }
        // Вторая часть тяжёлая.
        if messages.kind >= MessageKind::Error {
            return Ok(false);
        }

        for (rule, items) in rules_map {
            let res = match rule {
                STATUS_RULE_CANCEL => {
                    let pre_cancel_req = PreCancelPlansReq {
                        item_list: items.iter().cloned().map(Into::into).collect(),
                        section_id: Section::EstimatedCommissionNotRequired,
                    };
                    pre_cancel_plan(pre_cancel_req, self.db_pool.clone())
                        .await?
                        .messages
                }
                STATUS_RULE_CHANGE => {
                    let pre_change_req = PreChangeFormReq {
                        item_list: items.into_iter().map(Into::into).collect(),
                        section_id: Section::EstimatedCommissionNotRequired,
                    };
                    pre_change_form(pre_change_req, self.db_pool.clone())
                        .await?
                        .messages
                }
                // TODO: Add when rule approve is added. "STATUS_RULE_APPROVE"
                _ => continue,
            };
            messages.add_messages(res);
        }
        Ok(true)
    }
}

/// Обновление полей ППЗ/ДС.
///
/// NB. Обновленные сущности при изменении статусов отправляются на монолит.
#[async_trait::async_trait]
pub(crate) trait PlanCollectedUpdate {
    /// Мануальное обновление полей "смерженных" [`Plan`] и [`ContractAmendment`]
    async fn update(
        plans: Vec<PlanOrAmendment>,
        fields: &[&'static str],
        messages: &mut Messages,
        recorder: &mut Recorder,
        handler: ProcessingRulesChecker,
    ) -> Result<Vec<PlanOrAmendment>> {
        let (plans, amendments) = PlanOrAmendment::split_vec(plans);
        let (plans, amendments) = PlanOrAmendment::update_splitted(
            plans, amendments, fields, messages, recorder, handler,
        )
        .await?;

        Ok(PlanOrAmendment::collect(plans, amendments))
    }

    /// Мануальное обновление полей "смерженных" [`Plan`] и [`ContractAmendment`] с обновлением
    /// разных полей
    async fn update_different_fields(
        plans: Vec<PlanOrAmendment>,
        plan_fields: &[&'static str],
        amendment_fields: &[&'static str],
        messages: &mut Messages,
        recorder: &mut Recorder,
        handler: ProcessingRulesChecker,
    ) -> Result<Vec<PlanOrAmendment>> {
        let (plans, amendments) = PlanOrAmendment::split_vec(plans);
        let (plans, amendments) =
            PlanOrAmendment::update_splitted_different_fields(
                plans,
                amendments,
                plan_fields,
                amendment_fields,
                messages,
                recorder,
                handler,
            )
            .await?;

        Ok(PlanOrAmendment::collect(plans, amendments))
    }

    /// Мануальное обновление полей "разделенных" [`Plan`] и [`ContractAmendment`]
    async fn update_splitted(
        plans: Vec<Plan>,
        amendments: Vec<ContractAmendment>,
        fields: &[&'static str],
        messages: &mut Messages,
        recorder: &mut Recorder,
        handler: ProcessingRulesChecker,
    ) -> Result<(Vec<Plan>, Vec<ContractAmendment>)> {
        update_splitted_inner(
            plans, amendments, fields, fields, messages, recorder, handler,
        )
        .await
    }

    /// Мануальное обновление полей "разделенных" [`Plan`] и [`ContractAmendment`]
    #[allow(clippy::too_many_arguments)]
    async fn update_splitted_different_fields(
        plans: Vec<Plan>,
        amendments: Vec<ContractAmendment>,
        plan_fields: &[&'static str],
        amendment_fields: &[&'static str],
        messages: &mut Messages,
        recorder: &mut Recorder,
        handler: ProcessingRulesChecker,
    ) -> Result<(Vec<Plan>, Vec<ContractAmendment>)> {
        update_splitted_inner(
            plans,
            amendments,
            plan_fields,
            amendment_fields,
            messages,
            recorder,
            handler,
        )
        .await
    }
}

#[async_trait::async_trait]
#[allow(dead_code)]
pub(crate) trait PlanRepCollectedUpdate {
    /// Обновление полей, которые являются [`Option::Some`], для
    /// "смерженных" [`PlanRep`] и [`ContractAmendmentRep`]
    async fn update(
        plans: Vec<PlanOrAmendmentRep>,
        messages: &mut Messages,
        recorder: &mut Recorder,
        handler: ProcessingRulesChecker,
    ) -> Result<Vec<PlanOrAmendment>> {
        let (plans, amendments) = PlanOrAmendmentRep::split_vec(plans);
        let (plans, amendments) =
            Self::update_splitted(plans, amendments, messages, recorder, handler)
                .await?;

        Ok(PlanOrAmendment::collect(plans, amendments))
    }

    /// Обновление полей, которые являются [`Option::Some`], для
    /// "разделенных" [`PlanRep`] и [`ContractAmendmentRep`]
    async fn update_splitted(
        plans: Vec<PlanRep>,
        amendments: Vec<ContractAmendmentRep>,
        messages: &mut Messages,
        recorder: &mut Recorder,
        handler: ProcessingRulesChecker,
    ) -> Result<(Vec<Plan>, Vec<ContractAmendment>)> {
        // Here we do a sanity check for consistency of fields for the update.
        // We also collect the update fields because they are needed by the historial
        let (plan_mask, amendment_mask) = (
            PlanRep::create_strict_bind_mask(&plans)?,
            ContractAmendmentRep::create_strict_bind_mask(&amendments)?,
        );
        let (plan_update_fields, amendment_update_fields) = (
            update_fields_helper::<Plan>(&plan_mask),
            update_fields_helper::<ContractAmendment>(&amendment_mask),
        );

        let update_plans = plans
            .into_iter()
            .map(|x| x.into_item().map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;
        let update_amendments = amendments
            .into_iter()
            .map(|x| x.into_item().map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;

        update_splitted_inner(
            update_plans,
            update_amendments,
            &plan_update_fields,
            &amendment_update_fields,
            messages,
            recorder,
            handler,
        )
        .await
    }

    /// Мануальное обновление полей "смерженных" [`PlanRep`] и [`ContractAmendmentRep`]
    async fn update_manually(
        plans: Vec<PlanOrAmendmentRep>,
        fields: &[&'static str],
        messages: &mut Messages,
        recorder: &mut Recorder,
        handler: ProcessingRulesChecker,
    ) -> Result<Vec<PlanOrAmendment>> {
        let (plans, amendments) = PlanOrAmendmentRep::split_vec(plans);
        let (plans, amendments) = Self::update_splitted_manually(
            plans, amendments, fields, messages, recorder, handler,
        )
        .await?;

        Ok(PlanOrAmendment::collect(plans, amendments))
    }

    /// Мануальное обновление полей "разделенных" [`PlanRep`] и [`ContractAmendmentRep`]
    async fn update_splitted_manually(
        plans: Vec<PlanRep>,
        amendments: Vec<ContractAmendmentRep>,
        fields: &[&'static str],
        messages: &mut Messages,
        recorder: &mut Recorder,
        handler: ProcessingRulesChecker,
    ) -> Result<(Vec<Plan>, Vec<ContractAmendment>)> {
        let update_plans = plans
            .into_iter()
            .map(|x| x.into_item().map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;
        let update_amendments = amendments
            .into_iter()
            .map(|x| x.into_item().map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;

        update_splitted_inner(
            update_plans,
            update_amendments,
            fields,
            fields,
            messages,
            recorder,
            handler,
        )
        .await
    }
}

#[allow(clippy::too_many_arguments)]
async fn update_splitted_inner(
    plans: Vec<Plan>,
    amendments: Vec<ContractAmendment>,
    plan_fields: &[&'static str],
    amendment_fields: &[&'static str],
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
) -> Result<(Vec<Plan>, Vec<ContractAmendment>)> {
    let updated_plans = recorder
        .process_update_checked(plans, plan_fields, handler.clone(), messages)
        .await?;
    let updated_amendments = recorder
        .process_update_checked(amendments, amendment_fields, handler, messages)
        .await?;

    Ok((updated_plans, updated_amendments))
}

impl PlanCollectedUpdate for PlanOrAmendment {}
impl PlanRepCollectedUpdate for PlanOrAmendmentRep {}
