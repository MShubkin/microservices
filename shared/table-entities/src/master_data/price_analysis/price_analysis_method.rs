use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::Type;
use uuid::Uuid;

/// Метод ценообразования
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Type, Serialize, Deserialize, DbEnum,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum PriceAnalysisMethodId {
    /// Unknown (this is a hack to allow into)
    #[db_default]
    Undefined = 0,
    /// Метод сопоставимых рыночных цен (анализ рынка)
    MarketAnalysis = 1,
    /// Метод удельных показателей (параметрический)
    Parametric = 2,
    /// Затратный метод
    Cost = 3,
    ///Тарифный метод
    Tariff = 4,
    /// Проектно-сметный метод
    ProjectEstimate = 5,
    /// Метод расчета цены НИОКР
    RND = 6,
    /// Метод формирования цены с учетом внешних факторов
    External = 7,
    /// Метод формирования цены на товары для машиностроительной отрасли длительного производства
    LongTerm = 8,
}

/// Справочник "Метод ценообразования"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "price_analysis_method"]
pub struct PriceAnalysisMethod {
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: PriceAnalysisMethodId,
    /// uuid записи
    pub uuid: Uuid,
    /// Наименование метода
    #[serde(rename = "text")]
    pub name: String,
    /// Запись удалена
    pub is_removed: bool,
    /// Создано
    pub created_at: AsezTimestamp,
    /// Изменено
    pub changed_at: AsezTimestamp,
    /// Создатель
    pub created_by: i32,
    /// Кем изменено
    pub changed_by: i32,
}
