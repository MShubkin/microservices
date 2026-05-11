use asez2_shared_db::db_item::{AsezDate, AsezTimestamp, DbUpsert};
use asez2_shared_db::DbItem;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbAdaptor;
use uuid::Uuid;

use crate::maths::CurrencyValue;

use super::{DbItemDel, TcpDbItem};
use super::{TCPCheckStatus, TCPReviewResult, TcpGeneralStatus};

/// Заголовок ТКП
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "proposal_head"]
pub struct ProposalHeader {
    /// UID ТКП
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Номер ТКП
    #[item_field_autogen]
    pub id: i64,
    /// id ЭТП ГПБ.
    pub etp_id: Option<i32>,
    /// UUID ЗЦИ
    pub request_uuid: Uuid,
    /// UUID в иерархии файлов
    pub hierarchy_uuid: Option<Uuid>,
    /// UID организации
    pub supplier_uuid: Uuid,
    /// Начало срока действия
    pub start_date: Option<AsezDate>,
    /// Окончание срока действия
    pub end_date: Option<AsezDate>,
    /// Валюта
    pub currency_id: i32,
    /// Статус общий
    pub status_id: TcpGeneralStatus,
    /// Статус расммотрения
    pub status_check_id: TCPCheckStatus,
    /// Результат рассмотрения
    pub result_id: Option<TCPReviewResult>,
    /// Дата поступления ТКП
    pub receive_date: Option<AsezTimestamp>,
    /// Источник инфо об организации
    pub proposal_source: Option<String>,
    /// Стоимость организации
    #[adaptor_rename = "supplier_sum_excluded_vat_total"]
    pub sum_excluded_vat_total: Option<CurrencyValue>,
    /// Контактный телефон
    pub contact_phone: Option<String>,
    /// Создал
    pub created_by: i32,
    /// Дата создания
    pub created_at: AsezTimestamp,
    /// Изменил
    pub changed_by: i32,
    /// Дата изменения
    pub changed_at: AsezTimestamp,
}

impl DbItemDel for ProposalHeader {}
impl DbUpsert for ProposalHeader {}
impl TcpDbItem for ProposalHeader {}
