use asez2_shared_db::db_item::{AsezTimestamp, DbItemDel};
use asez2_shared_db::DbItem;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbAdaptor;
use uuid::Uuid;

use super::TcpDbItem;

#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "status_history"]
pub struct StatusHistory {
    /// Уникальный идентификатор записи истории
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Уникальный идентификатор объекта (ЗЦИ, ТКП)
    pub object_uuid: Uuid,
    /// Уникальный идентификатор объекта (ЗЦИ, ТКП)
    #[db_field_name = "tcp_status_type"]
    pub tcp_status_type_id: Option<i16>,
    /// Прошлый статус
    pub status_id: i16,
    /// Создал
    pub created_by: i32,
    /// Дата создания
    pub created_at: AsezTimestamp,
}

impl DbItemDel for StatusHistory {}
impl TcpDbItem for StatusHistory {}
