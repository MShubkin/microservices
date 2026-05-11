pub use crate::domain::enums::master_data::DirectoryType as NSIDictionaryKind;
use ahash::AHashMap;
pub use monolith_service::dto::dictionary::CommonDictionaryKind as PlanningCommonDictionaryKind;
pub use monolith_service::dto::dictionary::DictionaryKind as PlanningDictionaryKind;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;

/// Замена полей экспортируемых данных.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReplacementKind {
    /// Поиск по id в словаре NSI
    NSIDictionary {
        kind: NSIDictionaryKind,
        mapping: ValueMapping,
    },
    /// Поиск по id в справочнике Планирования
    PlanningDictionary {
        kind: PlanningDictionaryKind,
        mapping: ValueMapping,
    },
    /// Поиск по id в общем справочнике Планирования
    PlanningCommonDictionary {
        kind: PlanningCommonDictionaryKind,
        mapping: ValueMapping,
    },
    /// Прямая замена id на строку (для DbEnum типов)
    Enum { replacement: EnumReplacement },
    /// Замена boolean на строки
    Boolean {
        true_value: Cow<'static, str>,
        false_value: Cow<'static, str>,
    },
    /// Для AsezTimestamp, преобразование в AsezDate
    Date,
    /// Строковое значение статуса
    Status,
    /// Строковое значение статуса и его предка
    StatusWithParent,
    /// Размер списка
    Length,
    /// Список с разделителем
    JoinedList { separator: Cow<'static, str> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EnumReplacement {
    ApprovalStatus,
    EcAgendaStatus,
    EcProtocolStatus,
    CommissionKind,
    ExpertConclusionId,
    PlanStatus,
    /// Собственное определение замены id на строковое значение
    Other(Vec<(i64, String)>),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, derive_more::Deref)]
pub struct ReplacementConfig(AHashMap<String, ReplacementKind>);

impl<'a> FromIterator<(&'a str, ReplacementKind)> for ReplacementConfig {
    fn from_iter<T: IntoIterator<Item = (&'a str, ReplacementKind)>>(
        iter: T,
    ) -> Self {
        ReplacementConfig(
            iter.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        )
    }
}

/// Способ получения строкового значения по данным из словаря.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ValueMapping {
    /// Значение поля `text`.
    Text,
    /// Значение поля `text_short`.
    TextShort,
    /// Значение поля `code` (преобразованное в строку, если надо).
    Code,
    /// Конкатенация значений полей `code` и `text`.
    CodeText,
    /// Пользователь, в формате `Фамилия И.О.`.
    User,
}

#[macro_export]
macro_rules! replacement {
    ($field:ident: $($tt:tt)*) => {
        (stringify!($field), replacement!(@r $($tt)*))
    };

    (@r nsi_dict($kind:ident) as $mapper:ident) => {
        $crate::presentation::dto::export::ReplacementKind::NSIDictionary {
            kind: $crate::presentation::dto::export::NSIDictionaryKind::$kind,
            mapping: $crate::presentation::dto::export::ValueMapping::$mapper,
        }
    };

    (@r planning_dict($kind:ident) as $mapper:ident) => {
        $crate::presentation::dto::export::ReplacementKind::PlanningDictionary {
            kind: $crate::presentation::dto::export::PlanningDictionaryKind::$kind,
            mapping: $crate::presentation::dto::export::ValueMapping::$mapper,
        }
    };

    (@r planning_common_dict($kind:ident) as $mapper:ident) => {
        $crate::presentation::dto::export::ReplacementKind::PlanningCommonDictionary {
            kind: $crate::presentation::dto::export::PlanningCommonDictionaryKind::$kind,
            mapping: $crate::presentation::dto::export::ValueMapping::$mapper,
        }
    };

    (@r boolean($true_val:expr, $false_val:expr)) => {
        $crate::presentation::dto::export::ReplacementKind::Boolean {
            true_value: $true_val.into(),
            false_value: $false_val.into(),
        }
    };

    (@r enum($kind:ident)) => {
        $crate::presentation::dto::export::ReplacementKind::Enum {
            replacement: $crate::presentation::dto::export::EnumReplacement::$kind
        }
    };

    (@r enum_display($kind:ident)) => {
        $crate::presentation::dto::export::ReplacementKind::Enum {
            replacement: $crate::presentation::dto::export::EnumReplacement::Other(
                <$kind as asez2_shared_db::db_item::EnumDiscriminant>::DISCRIMINANTS
                    .iter()
                    .map(|(var, repr)| ((*repr).into(), var.to_string()))
                    .collect(),
            )
        }
    };

    (@r enum($($val:expr),* $(,)?)) => {
        $crate::presentation::dto::export::ReplacementKind::Enum {
            replacement: EnumReplacement::Other([$(($val as i64, $val.to_string())),*].into_iter().collect())
        }
    };

    (@r joined_list($sep:expr)) => {
        $crate::presentation::dto::export::ReplacementKind::JoinedList {
            separator: ($sep).into(),
        }
    };

    (@r $kind:ident) => {
        $crate::presentation::dto::export::ReplacementKind::$kind
    };
}

pub fn default_replacement_config(
) -> impl Iterator<Item = (&'static str, ReplacementKind)> {
    [
        // from Planning service search_by_id dictionary
        replacement!(pricing_expert_id: planning_dict(Users) as Text),
        replacement!(supplier_id: planning_dict(Organization) as Text),
        replacement!(created_by: planning_dict(Users) as Text),
        replacement!(changed_by: planning_dict(Users) as Text),
        replacement!(user_id: planning_dict(Users) as Text),
        replacement!(okved2_id: planning_dict(Okved2) as Text),
        replacement!(okpd2_id: planning_dict(Okpd2) as Text),
        replacement!(okato_id: planning_dict(Okato) as Text),
        // НСИ (Master Data Service)`
        replacement!(
            agenda_status_id: nsi_dict(EstimatedCommissionAgendaStatus) as Text
        ),
        replacement!(expert_conclusion_id: nsi_dict(ExpertConclusionType) as Text),
        replacement!(pricing_method_id: nsi_dict(PriceAnalysisMethod) as Text),
        replacement!(pricing_organization_unit_id: nsi_dict(PricingUnit) as Text),
        // Planning service master_data/updates/x
        replacement!(category_id: planning_common_dict(Category) as Text),
        replacement!(country_id: planning_common_dict(Country) as Text),
        replacement!(currency_id: planning_common_dict(Currency) as Text),
        replacement!(customer_id: planning_common_dict(Customer) as Text),
        replacement!(department_id: planning_common_dict(Department) as Text),
        replacement!(
            purchasing_method_id: planning_common_dict(PurchasingMethod) as Text
        ),
        replacement!(section_id: planning_common_dict(Section) as Text),
        replacement!(unit_id: planning_common_dict(Unit) as Text),
        replacement!(vat_id: planning_common_dict(Vat) as Text),
        replacement!(
            single_supplier_reason_id: planning_common_dict(PurchasingPolicyItem)
                as Text
        ),
        replacement!(
            contract_amendment_types: planning_common_dict(ContractAmendmentKind)
                as Text
        ),
        // status is fetched from monolith and printed prepended with its parent
        replacement!(status_id: StatusWithParent),
        // enum
        replacement!(
            agenda_status_id: enum(EcAgendaStatus)
        ),
        replacement!(
            protocol_status_id: enum(EcProtocolStatus)
        ),
        replacement!(
            commission_kind_id: enum(CommissionKind)
        ),
        replacement!(
            expert_conclusion_id: enum(ExpertConclusionId)
        ),
    ]
    .into_iter()
}
