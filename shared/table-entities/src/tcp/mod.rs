mod organization_question;
mod proposal_header;
mod proposal_item;
mod request_header;
mod request_item;
mod request_partner;
mod status_history;
mod subject_purchased;

use asez2_shared_db::db_item::{DbItemDel, Filter, Select};
use asez2_shared_db::result::SharedDbError;
use asez2_shared_db::DbItem;
use asez2_shared_db::{impl_join_on, joined};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::{PgPool, Transaction, Type};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

pub use organization_question::*;
pub use proposal_header::*;
pub use proposal_item::*;
pub use request_header::*;
pub use request_item::*;
pub use request_partner::*;
pub use status_history::*;
pub use subject_purchased::*;

impl_join_on!(RequestHeader:uuid => RequestItem:request_uuid, aggr);
impl_join_on!(RequestHeader:uuid => RequestPartner:request_uuid, aggr);
impl_join_on!(RequestPartner:uuid => ProposalHeader:supplier_uuid, aggr);
impl_join_on!(RequestPartner:uuid => OrganizationQuestion:supplier_uuid, aggr);
impl_join_on!(ProposalHeader:uuid => ProposalItem:proposal_uuid, aggr);
impl_join_on!(ProposalHeader:supplier_uuid => RequestPartner:uuid);
impl_join_on!(ProposalHeader:supplier_uuid => RequestPartner:uuid, aggr);
impl_join_on!(ProposalItem:request_item_uuid => RequestItem:uuid);
impl_join_on!(ProposalHeader:request_uuid => RequestHeader:uuid);

// Выборка детальной информации ЗЦИ
joined!(
    !JoinedPriceInformationInfoDetail,
    header: RequestHeader,
    items: RequestItem[RequestHeader => RequestItem, aggr],
    suppliers: RequestPartner[RequestHeader => RequestPartner, aggr],
    proposal_headers: ProposalHeader[RequestPartner => ProposalHeader, aggr],
    organization_questions: OrganizationQuestion[RequestPartner => OrganizationQuestion, aggr],
);

// Упрощённая выборка по ЗЦИ, в основном для pre_request/request_price_info_close
joined!(
    !RequestWithPartners,
    header: RequestHeader,
    suppliers: RequestPartner[RequestHeader => RequestPartner, aggr],
);

// Упрощённая выборка по ЗЦИ, в основном для pre_request/request_price_info_close
joined!(
    !BasicRequestDetails,
    header: RequestHeader,
    suppliers: RequestPartner[RequestHeader => RequestPartner, aggr],
    items: RequestItem[RequestHeader => RequestItem, aggr],
);

joined!(
    !ProposalWithPartners,
    header: ProposalHeader,
    supplier: RequestPartner[ProposalHeader => RequestPartner],
);

joined!(
    !ProposalWithItems,
    header: ProposalHeader,
    items: ProposalItem[ProposalHeader => ProposalItem, aggr],
);

// Выборка данных по техническим предложением по "get/proposal_detail".
joined!(
    !GetProposalDetailData,
    proposal_header: ProposalHeader,
    request_header: RequestHeader[ProposalHeader => RequestHeader],
    partner: RequestPartner[ProposalHeader => RequestPartner],
    items: ProposalItem[ProposalHeader => ProposalItem, aggr],
);

// Иногда ищем на оборот.
joined!(
    !PartnerWithProposals,
    partner: RequestPartner,
    proposals: ProposalHeader[RequestPartner => ProposalHeader, aggr],
);

// Выборка атрибутов позиции ЗЦИ по ТКП
joined!(
    proposal_item: ProposalItem,
    request_item: RequestItem[ProposalItem => RequestItem],
);

/// Тип ЗЦИ
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
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum PriceInformationRequestType {
    #[db_default]
    /// Не определён
    Undefined = 0,
    /// Открытый
    Public = 1,
    /// Закрытый
    Private = 2,
}

/// Системный статус ЗЦИ
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
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum PriceInformationRequestStatus {
    #[db_default]
    /// Не определён
    Undefined = 0,
    /// Создан
    Created = 10,
    /// Сбор предложений
    Received = 20,
    /// "Проект ТКП"
    TcpProject = 70,
    /// "Передан на ЭТП"
    TransferredToEtp = 80,
    /// "Приём ТКП"
    AcceptingIncomingTCPs = 90,
    /// "Ошибка передачи на ЭТП (Электронная торговая площадка)"
    TransferToEtpError = 100,
    /// "Приём закрыт"
    EntryClosed = 110,
    /// "Прием закрыт досрочно"
    EntryClosedEarly = 120,
    /// "Рассмотрено"
    Reviewed = 130,
    /// "Удалено"
    Deleted = 140,
    /// "Ошибка публикации изменений"
    ErrorPublishingChanges = 150,
}

impl Display for PriceInformationRequestStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => write!(f, "Не определён"),
            PriceInformationRequestStatus::Created => write!(f, "Создан"),
            PriceInformationRequestStatus::Received => {
                write!(f, "Сбор предложений")
            }
            PriceInformationRequestStatus::TcpProject => write!(f, "Проект ТКП"),
            PriceInformationRequestStatus::TransferredToEtp => {
                write!(f, "Передан на ЭТП")
            }
            PriceInformationRequestStatus::AcceptingIncomingTCPs => {
                write!(f, "Приём ТКП")
            }
            PriceInformationRequestStatus::TransferToEtpError => {
                write!(f, "Ошибка передачи на ЭТП (Электронная торговая площадка)")
            }
            PriceInformationRequestStatus::EntryClosed => write!(f, "Приём закрыт"),
            PriceInformationRequestStatus::EntryClosedEarly => {
                write!(f, "Прием закрыт досрочно")
            }
            PriceInformationRequestStatus::Reviewed => write!(f, "Рассмотрено"),
            PriceInformationRequestStatus::Deleted => write!(f, "Удалено"),
            PriceInformationRequestStatus::ErrorPublishingChanges => {
                write!(f, "Ошибка публикации изменений")
            }
        }
    }
}

/// Пользовательские статусы ЗЦИ по поставщикам
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
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum SupplierStatus {
    #[db_default]
    /// Не определён
    Undefined = 0,
    /// Направлено поставщику
    SentToSupplier = 1,
    /// Получено поставщиком
    ObtainedBySupplier = 2,
    /// Отказ поставщика
    SupplierRefusal = 3,
    /// Идет подготовка ТКП
    TkpIsBeingPrepared = 4,
    /// ТКП получено
    TkpReceived = 5,
    /// ТКП проверено
    TkpVerified = 6,
    /// ТКП отклонено
    TkpRejected = 7,
    /// ТКП отсутствует
    TkpIsMissing = 8,
}

/// Системный статус ТКП
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
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum TcpGeneralStatus {
    #[db_default]
    /// Не определён
    Undefined = 0,
    /// ТКП Создано
    Created = 10,
    /// ТКП Получено
    Received = 20,
    /// ТКП Удалено
    Deleted = 25,
}

/// Cтатус рассмотрения ТКП
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
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum TCPCheckStatus {
    #[db_default]
    /// Не определён
    Undefined = 0,
    /// Рассмотрение
    Review = 30,
    /// Рассмотрено
    Reviewed = 40,
}

/// Результат рассмотрения ТКП
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
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum TCPReviewResult {
    #[db_default]
    /// Не определён
    Undefined = 0,
    /// Учитывать при АЦ
    Consider = 50,
    /// Не учитывать при АЦ
    Ignore = 60,
}

/// Источник Информации
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
#[repr(i16)]
pub enum TcpSource {
    #[db_default]
    /// Не определён
    Undefined = 0,
    /// ЭТП ГПБ
    EtpGpb = 1,
    /// Ручное создание
    Mannual = 2,
}

/// This trait contains "convenience" functions which are little less verbose than
/// using the standard TCP items.
/// Во избежаний конфликта с типажем DbItem (а он будет иначе), немного изменены
/// наименования функций (так проще).
#[async_trait::async_trait]
pub trait TcpDbItem: DbItemDel {
    async fn insert_ret(
        mut self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<Self, SharedDbError> {
        self.insert_returning(tx).await
    }
    async fn update_ret(
        self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<Self, SharedDbError> {
        let mut fields = Self::FIELDS.to_vec();
        fields.retain(|field| *field != "id");
        self.update_returning::<_, &str>(Some(&fields), None, tx).await
    }
    async fn get_by_uuid(uuid: Uuid, pool: &PgPool) -> Result<Self, SharedDbError> {
        let select = Select::full::<Self>().in_any("uuid", [uuid]);
        Self::select_option(&select, pool)
            .await?
            .ok_or(format!("Record with uuid={uuid} not found.").into())
    }
    async fn get_by_request_uuids(
        uuids: &[Uuid],
        pool: &PgPool,
    ) -> Result<Vec<Self>, SharedDbError> {
        let select =
            Select::full::<Self>().in_any("request_uuid", uuids).take_first();
        Self::select(&select, pool).await
    }
    async fn delete_by_uuids(
        uuids: &[Uuid],
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<usize, SharedDbError> {
        let filter = Filter::in_any("uuid", uuids).into();
        Self::delete_returning(&filter, tx).await.map(|r| r.len())
    }
}

macro_rules! impl_numerate {
    ($entity_name:ident) => {
        impl $entity_name {
            pub fn numerate(mut self, n: &mut i16) -> Self {
                self.number = *n;
                *n += 1;
                self
            }
        }
    };
}
impl_numerate!(ProposalItem);
impl_numerate!(RequestItem);
impl_numerate!(RequestPartner);
