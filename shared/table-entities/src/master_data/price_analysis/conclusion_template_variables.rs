use crate::master_data::price_analysis::sample_conclusion::ConclusionTemplateTypeId;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::DbItem;
use serde::{Deserialize, Serialize};
use shared_db_derive::{DbAdaptor, DbEnum};
use sqlx::Type;
use uuid::Uuid;

/// Операция группирования (позиции)
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
pub enum ConclusionTemplateVariablesGrpOperId {
    #[db_default]
    Undefined = 0,
    Sum = 1,
    Min = 2,
    Max = 3,
    Concat = 4,
    Collect = 5,
}

/// Справочник "Переменные шаблонов заключений"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "conclusion_templates_variables"]
pub struct ConclusionTemplateVariables {
    /// Id записи
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: i16,
    /// Идентификатор записи в таблице
    pub uuid: Uuid,
    /// ID шаблона
    pub template_id: i16,
    /// Id типа шаблона
    pub template_type_id: ConclusionTemplateTypeId,
    /// Код переменной
    pub var_code: String,
    /// Название переменной
    pub var_name: String,
    /// Тип данных
    pub data_type: String,
    /// Длина (число знаков)
    pub leng: i32,
    /// Число десятичных разрядов
    pub decimals: i32,
    /// Имя внутреннего источника
    pub it_name: String,
    /// Поле внутреннего источника
    pub it_field: String,
    /// Формула вычисления с именами полей внутреннего источника
    pub it_formula: String,
    /// Операция группирования (позиции)
    pub grp_oper: ConclusionTemplateVariablesGrpOperId,
    /// Внешний источник данных
    pub ext_data: String,
    /// Запись удалена
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}
