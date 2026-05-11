use asez2_shared_db::db_item::*;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::Type;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Hash,
    Eq,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
    derive_more::Display,
)]
#[repr(i16)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Не задано
    #[db_default]
    Undefined = 0,

    #[display(fmt = "Назначение Эксперта АЦ")]
    LottingCompleted351 = 100,
    #[display(fmt = "Проведение АЦ")]
    LottingCompleted352 = 101,
    #[display(fmt = "Завершение АЦ")]
    LottingCompleted353 = 102,

    #[display(fmt = "Проведение АЦ повторно")]
    EcExpertAppointmentRepeat = 300,

    #[display(fmt = "Согласование ПД для Руководителя")]
    SdAgreementDirector = 400,
    #[display(fmt = "Согласование ПД для Эксперта")]
    SdAgreementExpert = 401,
    #[display(fmt = "Согласование ПД для Исполнителя")]
    SdAgreementExecutor = 402,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[repr(i16)]
#[serde(try_from = "i16", into = "i16")]
pub enum NotificationGroupId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    PriceAnalysis = 1,
    TechnicalCommercialProposal = 2,
    EstimatedCommission = 3,
    SpecializedDepartments = 4,
}

#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "notification"]
/// Информация об уведомлении, которое может быть отправлено пользователю
pub struct Notification {
    /// Тип уведомления
    #[item_field_pkey]
    pub id: NotificationType,
    /// Id группы, по которой направляется уведомление
    pub group_id: NotificationGroupId,
    /// Наименование уведомления
    pub name: String,
    /// Тема письма
    ///
    /// Содержит темплейт с именами полей других сущностей,
    /// которые могут быть подставлены
    pub subject: String,
    /// Тело письма
    ///
    /// Содержит темплейт с именами полей других сущностей,
    /// которые могут быть подставлены
    pub body: String,
    /// Признак удаления
    pub is_removed: bool,
    /// Дата и время создания
    pub created_at: AsezTimestamp,
    /// Код пользователя, который создал
    pub created_by: i32,
    /// Дата и время изменения
    pub changed_at: AsezTimestamp,
    /// Код пользователя, который изменил
    pub changed_by: i32,
}

/// Настройки уведомлений пользователя
#[derive(Debug, Default, Clone, DbItem, PartialEq, Serialize, Deserialize)]
#[item_table = "user_notification_settings"]
pub struct UserNotificationSettings {
    /// Айди пользователь
    #[item_field_pkey]
    pub user_id: i32,
    /// Признак отключения всех уведомлений
    pub is_all_disable: bool,
    /// Массив типов уведомлений, которые пользователь отключил.
    pub black_list: Vec<NotificationType>,
    /// Дата и время создания
    pub created_at: AsezTimestamp,
    /// Код пользователя, который создал
    pub created_by: i32,
    /// Дата и время изменения
    pub changed_at: AsezTimestamp,
    /// Код пользователя, который изменил
    pub changed_by: i32,
}
