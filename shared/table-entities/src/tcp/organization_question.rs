use serde::{Deserialize, Serialize};
use uuid::Uuid;

use asez2_shared_db::db_item::{AsezTimestamp, DbItemDel};
use asez2_shared_db::DbItem;
use shared_db_derive::DbAdaptor;

use super::TcpDbItem;

/// Документы объектов
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "organization_question"]
pub struct OrganizationQuestion {
    /// Идентификатор вопроса
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Текст вопроса
    pub question_text: String,
    /// Идентификатор ответа
    pub answer_uuid: Option<Uuid>,
    /// Текст ответа
    pub answer_question_text: Option<String>,
    /// UID ЗЦИ
    pub request_uuid: Uuid,
    /// UID организации
    pub supplier_uuid: Uuid,
    /// Айди организации
    pub supplier_id: i32,
    /// Создал ответ
    pub created_by: i32,
    /// Дата создания вопроса
    pub question_created_at: AsezTimestamp,
    /// Дата создания ответа
    pub answer_created_at: Option<AsezTimestamp>,
    /// Дата публикации ответа
    pub answer_published_at: Option<AsezTimestamp>,
}

impl DbItemDel for OrganizationQuestion {}
impl TcpDbItem for OrganizationQuestion {}
