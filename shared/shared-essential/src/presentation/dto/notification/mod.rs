//! Модуль описывает контракты общения с `Notification` сервисом
use asez2_tables::view_storage::notification::NotificationType;
use serde::{Deserialize, Serialize};

pub mod estimated_commission;
pub use estimated_commission::*;

pub mod price_analysis;
pub use price_analysis::*;

pub mod specialized_departments;
pub use specialized_departments::*;

pub mod error;
pub use error::{NotificationError, NotificationResult};

/// Запрос на отправление уведомлений
#[derive(Deserialize, Serialize, Debug)]
pub struct SendNotificationReq {
    /// Список уведомлений
    pub notifications: Vec<NotificationReq>,
    /// Токен для взаимодействия с монолитом для получения
    /// доп данных
    pub token: String,
    /// Айди пользователя, который инициировал запрос
    pub user_id: i32,
}

/// Общий запрос на отправку уведомлений, который предполагает
/// отправку общего массива уведомлений каждому пользователю с подменой
/// данных, которые специфичны для каждого (имя, роль)
#[derive(Deserialize, Serialize, Debug)]
pub struct NotificationReq {
    /// Идентификаторы пользователей, которые должны будут получить
    /// уведомления
    pub receivers: Vec<Receiver>,
    /// Идентификаторы пользователей, которым будет отправлена копия уведомлений
    pub copy_to: Vec<Receiver>,
    /// Уведомления, которые будут отправлены каждому пользователю
    /// из [`Self::receivers`].
    pub notifications: Vec<NotificationBody>,
    /// Приложения, которые будут отправлены каждому пользователю
    /// из [`Self::receivers`].
    pub attachments: Option<Vec<AttachmentReq>>,
}

/// Получатель уведомления
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Receiver {
    /// Если данные по пользователю есть
    Other {
        id: i32,
        name: String,
        email: String,
    },
    /// Если же данные по пользователю находятся на стороне
    /// монолит
    UserFromMonolith { id: i32 },
}

/// Отправляемое приложение
#[derive(Deserialize, Serialize, Debug)]
pub struct AttachmentReq {
    /// Формат файла
    pub mime: String,
    /// Содержимое файла в base64
    pub payload: String,
    /// Название файла
    pub filename: String,
}

/// Общий набор возможных уведомлений
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "body", rename_all = "snake_case")]
pub enum NotificationBody {
    /// Повтороное назначение эксперта АЦ по ППЗ/ДС
    EcExpertAppointmentRepeat(EcExpertAppointmentRepeat),

    /// Завершение "лотирования" со статусом 351 (АЦ МТР. Назначение исполнителя)
    LottingCompleted351(LottingCompleted351),
    /// Завершение "лотирования" со статусом 352 (АЦ МТР. Исполнитель назначен)
    LottingCompleted352(LottingCompleted352),
    /// Завершение "лотирования" со статусом 353 (АЦ МТР. Анализ проведен)
    LottingCompleted353(LottingCompleted353),

    SdAgreementDirector(SdAgreementDirector),
    SdAgreementExpert(SdAgreementExpert),
    SdAgreementExecutor(SdAgreementExecutor),
}

impl NotificationBody {
    /// Соответствие [`NotificationReq`] варианту [`NotificationType`]
    pub fn ty(&self) -> NotificationType {
        use NotificationType::*;
        match self {
            Self::EcExpertAppointmentRepeat(_) => EcExpertAppointmentRepeat,

            Self::LottingCompleted351(_) => LottingCompleted351,
            Self::LottingCompleted352(_) => LottingCompleted352,
            Self::LottingCompleted353(_) => LottingCompleted353,

            Self::SdAgreementDirector(_) => SdAgreementDirector,
            Self::SdAgreementExpert(_) => SdAgreementExpert,
            Self::SdAgreementExecutor(_) => SdAgreementExecutor,
        }
    }
}

impl Receiver {
    pub fn id(&self) -> i32 {
        match self {
            Receiver::Other { id, .. } => *id,
            Receiver::UserFromMonolith { id } => *id,
        }
    }
}

/// Ответ на [запрос отправления уведомления](SendNotificationReq)
#[derive(Serialize, Deserialize, Debug)]
pub struct SendNotificationResponse {
    /// Статус, отправилось ли уведомление
    pub status: bool,
}
