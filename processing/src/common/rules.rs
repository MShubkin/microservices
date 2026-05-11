//! Модуль работает с правилами перехода статусов ППЗ.
use super::Result;

use asez2_shared_db::db_item::Select;
use asez2_shared_db::DbItem;
use shared_essential::domain::tables::legacy::plans::PlanStatus;

use ahash::AHashMap;
use sqlx::{Executor, Postgres};

pub(crate) const STATUS_RULE_CANCEL: &str = "pre_request_cancel";
pub(crate) const STATUS_RULE_CHANGE: &str = "pre_request_change_form";
/// We will need this once the code is written.
#[allow(dead_code)]
pub(crate) const STATUS_RULE_APPROVE: &str = "pre_request_approve";

/// Сущность которая содержит все правила.
#[derive(Debug)]
pub(crate) struct ProcessingRules {
    /// Правила перехода статусов ППЗ/ДС
    status_rules: NextStatusRules,
}

impl ProcessingRules {
    /// Правила не меняются.
    pub(crate) fn status_rules(&self) -> &NextStatusRules {
        &self.status_rules
    }

    pub(crate) async fn new<'a, Ex>(conn: Ex) -> Result<Self>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        Ok(Self {
            status_rules: NextStatusRules::load_rules(conn).await?,
        })
    }
}

/// Описывает переход статусов
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct StatusTransition {
    current: PlanStatus,
    /// Пустое значение разрешает любой переход
    next: Option<PlanStatus>,
}

impl StatusTransition {
    pub(crate) fn new(current: PlanStatus, next: Option<PlanStatus>) -> Self {
        Self { current, next }
    }
}

/// Завернутая HashMap чтобы проще проверять разрешается переход статуса или нет.
#[derive(Debug)]
pub(crate) struct NextStatusRules(AHashMap<StatusTransition, String>);

/// A "convenience trait" so we don't have do useless wrapping of a hash
impl NextStatusRules {
    pub(crate) fn get_rule(
        &self,
        current: PlanStatus,
        desired: PlanStatus,
    ) -> Option<&str> {
        let rule1 = StatusTransition::new(current, Some(desired));
        let rule2 = StatusTransition::new(current, None);

        self.0.get(&rule1).or_else(|| self.0.get(&rule2)).map(|x| x.as_ref())
    }

    pub(crate) async fn load_rules<'a, Ex>(conn: Ex) -> Result<Self>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        let select = Select::full::<PlanStatusRule>();
        let rules = PlanStatusRule::select(&select, conn)
            .await?
            .into_iter()
            .map(
                |PlanStatusRule {
                     status_id,
                     next_status_id,
                     rule,
                 }| {
                    let transition =
                        StatusTransition::new(status_id, next_status_id);
                    (transition, rule)
                },
            )
            .collect::<AHashMap<_, _>>();

        Ok(NextStatusRules(rules))
    }

    #[cfg(test)]
    pub(crate) fn transition_is_ok(
        &self,
        current: PlanStatus,
        desired: PlanStatus,
    ) -> bool {
        self.get_rule(current, desired).is_some()
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Debug, PartialEq, DbItem)]
#[item_table = "plan_status_transition"]
// Exists purely for boilerplate of retrieving the rules.
pub(crate) struct PlanStatusRule {
    #[item_field_pkey]
    status_id: PlanStatus,
    next_status_id: Option<PlanStatus>,
    rule: String,
}
