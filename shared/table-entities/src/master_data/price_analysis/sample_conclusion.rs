use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{impl_join_on, joined, DbAdaptor, DbItem};
use shared_db_derive::DbEnum;

use super::sample_conclusion_crit::SampleConclusionCrit;
use super::sample_conclusion_status::SampleConclusionStatusId;
use crate::master_data::price_analysis::conclusion_template_variables::ConclusionTemplateVariables;

impl_join_on!(SampleConclusion:id => ConclusionTemplateVariables:template_id, aggr);
impl_join_on!(SampleConclusion:id => SampleConclusionCrit:template_id, aggr);

joined!(
    template: SampleConclusion,
    crits: SampleConclusionCrit[SampleConclusion => SampleConclusionCrit, aggr],
    vars: ConclusionTemplateVariables[SampleConclusion => ConclusionTemplateVariables, aggr]
);

/// Id Типа шаблона заключения
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum ConclusionTemplateTypeId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// ППЗ
    Plan = 1,
    /// ДС
    ContractAmendment = 2,
}

/// Справочник "Шаблоны заключений"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "sample_conclusion"]
pub struct SampleConclusion {
    /// Номер шаблона
    #[item_field_autogen]
    pub id: i16,
    /// Идентификатор шаблона заключения
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Доступ к шаблону заключения
    pub access_id: SampleConclusionAccessId,
    /// Статус шаблона
    pub status_id: SampleConclusionStatusId,
    /// Текст шаблона
    pub text: String,
    /// Запись удалена
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum SampleConclusionAccessId {
    /// Индивидуальный
    #[db_default]
    Individual = 1,
    /// Общий
    Common = 2,
}
