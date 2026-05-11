use std::marker::PhantomData;
use std::str::FromStr;

use super::currency::Currency;
use super::time::PlanningTimestamp;
use super::{category::Category, customer::MonolithCustomer, unit::Unit, vat::Vat};
use crate::dto::purchasing_trend::PurchasingTrend;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};

/// Тип справочника, к которому монолит предоставляет доступ через
/// `/api/json/master_data/get_updates/<timestamp>/`,
/// см. http://rcdevstand.inlinegroup.ru/swagger/#/%D0%A1%D0%BF%D1%80%D0%B0%D0%B2%D0%BE%D1%87%D0%BD%D0%B8%D0%BA%D0%B8%20%D0%9C%D0%BE%D0%BD%D0%BE%D0%BB%D0%B8%D1%82%D0%B0.%20%D0%9A%D1%8D%D1%88%D0%B8%D1%80%D0%BE%D0%B2%D0%B0%D0%BD%D0%B8%D0%B5%20%D1%81%D0%BF%D1%80%D0%B0%D0%B2%D0%BE%D1%87%D0%BD%D0%B8%D0%BA%D0%BE%D0%B2%20%D0%BC%D0%BE%D0%BD%D0%BE%D0%BB%D0%B8%D1%82%D0%B0/post_api_json_master_data_get_updates__timestamp__
///
/// Внимание! В данных монолита встречаются позиции с отсутствующими полями "id" и "text".
/// Справочники с "кривыми" данными закомментированы.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Hash, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum CommonDictionaryKind {
    Category,
    ContractAmendmentItemKind,
    ContractAmendmentKind,
    Country,
    Currency,
    Customer,
    Department,
    FundingSource,
    MasterSystem,
    NomenclatureGroup,
    Organization,
    PublicationType,
    PurchasingMethod,
    PurchasingPolicyItem,
    PurchasingTrend,
    QualificationActivity,
    QualificationPlanDefault,
    QualificationPlanIrrelevant,
    RegulationDocument,
    Section,
    SmbException,
    Status,
    Unit,
    Vat,
    BudgetItem,
    ContractAmendmentApprovingDecision,
    CountryPP2013,
    Law2013Country,
    Law2013Okpd2,
    Law2013ViolationReason,
    MtrGroup,
    MtrReestr,
    MtrReestrName,
    MtrReestrProducerName,
    Okpd2PP2013,
    PaymentBalanceItem,
    PurchasingPolicy,
    Qualification,
    RepairStage,
    RfProductsReason,
    Users,
    #[serde(other)]
    Unknown,
}

/// Тип справочника, к которому монолит предоставляет доступ через
/// `/api/json/search/<name>/` и `/api/json/search_by_id/<name>/`,
/// см. http://rcdevstand.inlinegroup.ru/swagger/#/%D0%A1%D0%BF%D1%80%D0%B0%D0%B2%D0%BE%D1%87%D0%BD%D0%B8%D0%BA%D0%B8%20%D0%9C%D0%BE%D0%BD%D0%BE%D0%BB%D0%B8%D1%82%D0%B0.%20%D0%94%D0%BE%D1%81%D1%82%D1%83%D0%BF%20%D0%BF%D0%BE%20API
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Hash, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum DictionaryKind {
    Users,
    Organization,
    Okpd2,
    Okved2,
    Okato,
}

/// Список записей справочника монолита. Существует для получения данных в виде
///
/// ```ignore
/// "variant_name": [Dictionary]
/// ```
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub enum CommonDictionaryList {
    Customer(Vec<MonolithCustomer>),
    Unit(Vec<Unit>),
    Currency(Vec<Currency>),
    PurchasingTrend(Vec<PurchasingTrend>),
    Vat(Vec<Vat>),
    Category(Vec<Category>),

    /// TODO: заглушка на то, чтобы была пока возможность
    /// принимать любой справочник
    ///
    /// Используется `#[serde(untagged)]` из за того, что
    /// `#[serde(other)]` не подружить с `#[serde(flatten)]`
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

pub struct DictionaryJsonEntry {}

/// Структура элемента запроса справочника по его id
///
/// T дженерик используется, так как id может быть как i32, так и &str
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DictionaryRequestItem<T> {
    pub id: T,
}

/// Структура запроса справочников по API /api/json/*dictionary*/search/
#[derive(Default, Debug, Serialize)]
pub struct SearchRequest {
    // С какой записи делать выдачу
    pub from: u32,
    // Количество записей в выдаче
    pub quantity: u32,
    // Строка запроса
    pub search: String,
}

/// Представление массива возвращемых от монолита справочников
#[derive(Serialize, Deserialize, Debug)]
pub struct DictionaryListRes<T> {
    pub value: Vec<T>,
}

impl<T> Default for DictionaryListRes<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
        }
    }
}

/// Общий ответ на /get_updates/
#[derive(Deserialize, Debug)]
pub struct GetUpdatesResponse {
    pub changed_at: PlanningTimestamp,
    pub r#type: String,
    pub entities: Vec<UpdatedRecord>,
}

/// Общий ответ на /get_updates/
#[derive(Deserialize, Debug)]
pub struct GetUpdatesJsonResponse<T> {
    pub changed_at: PlanningTimestamp,
    pub r#type: String,
    pub entities: Vec<GetUpdatesJsonRecord<T>>,
}

/// Общий элемент ответа на /get_updates/
///
/// Гарантирует, что каждому `dictionary_kind` соответствуют
/// `items` с записями из этого справочника.
///
/// Удовлетворяет структуре
/// ```ignore
/// {
///     "changed_at": 12345,
///     "[Наименование справочника]": [Dictionary]
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct UpdatedRecord {
    pub changed_at: PlanningTimestamp,
    #[serde(flatten, default)]
    pub items: CommonDictionaryList,
}

/// Общий элемент ответа на /get_updates/
///
/// Гарантирует, что каждому `dictionary_kind` соответствуют
/// `items` с записями из этого справочника.
///
/// Удовлетворяет структуре
/// ```ignore
/// {
///     "changed_at": 12345,
///     "[Наименование справочника]": [Dictionary]
/// }
/// ```
#[derive(Debug)]
pub struct GetUpdatesJsonRecord<T> {
    pub changed_at: PlanningTimestamp,
    pub kind: CommonDictionaryKind,
    pub items: Vec<T>,
}

impl<'de, T> Deserialize<'de> for GetUpdatesJsonRecord<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UpdatesVisitor<T>(PhantomData<T>);
        impl<T> UpdatesVisitor<T> {
            fn new() -> Self {
                UpdatesVisitor(Default::default())
            }
        }

        impl<'de, T: Deserialize<'de>> Visitor<'de> for UpdatesVisitor<T> {
            type Value = GetUpdatesJsonRecord<T>;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                formatter.write_str("dictionary entry")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                #[derive(Deserialize)]
                enum Key {
                    #[serde(rename = "changed_at")]
                    ChangedAt,
                    #[serde(untagged)]
                    Dictionary(CommonDictionaryKind),
                }

                let mut changed_at = None;
                let mut kind_items = None;

                while let Some(key) = map.next_key::<Key>()? {
                    match key {
                        Key::ChangedAt => {
                            if changed_at.replace(map.next_value()?).is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "changed_at",
                                ));
                            }
                        }
                        Key::Dictionary(kind) => {
                            if kind_items
                                .replace((kind, map.next_value()?))
                                .is_some()
                            {
                                return Err(serde::de::Error::duplicate_field(
                                    "<dictionary>",
                                ));
                            }
                        }
                    }
                }
                let Some(changed_at) = changed_at else {
                    return Err(serde::de::Error::missing_field("changed_at"));
                };
                let Some((kind, items)) = kind_items else {
                    return Err(serde::de::Error::missing_field("<dictionary>"));
                };
                Ok(GetUpdatesJsonRecord {
                    changed_at,
                    kind,
                    items,
                })
            }
        }

        deserializer.deserialize_map(UpdatesVisitor::new())
    }
}

impl FromStr for CommonDictionaryKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::CommonDictionaryKind::*;

        let val = match s {
            "Category" => Category,
            "ContractAmendmentItemKind" => ContractAmendmentItemKind,
            "ContractAmendmentKind" => ContractAmendmentKind,
            "Country" => Country,
            "Currency" => Currency,
            "Customer" => Customer,
            "Department" => Department,
            "FundingSource" => FundingSource,
            "MasterSystem" => MasterSystem,
            "NomenclatureGroup" => NomenclatureGroup,
            "Organization" => Organization,
            "PublicationType" => PublicationType,
            "PurchasingMethod" => PurchasingMethod,
            "PurchasingPolicyItem" => PurchasingPolicyItem,
            "PurchasingTrend" => PurchasingTrend,
            "RegulationDocument" => RegulationDocument,
            "Section" => Section,
            "SmbException" => SmbException,
            "Status" => Status,
            "Unit" => Unit,
            "Vat" => Vat,
            "BudgetItem" => BudgetItem,
            "ContractAmendmentApprovingDecision" => {
                ContractAmendmentApprovingDecision
            }
            "PaymentBalanceItem" => PaymentBalanceItem,
            "PurchasingPolicy" => PurchasingPolicy,
            "Users" => Users,
            "QualificationActivity" => QualificationActivity,
            "QualificationPlanDefault" => QualificationPlanDefault,
            "QualificationPlanIrrelevant" => QualificationPlanIrrelevant,
            "Qualification" => Qualification,
            "RepairStage" => RepairStage,
            "RfProductsReason" => RfProductsReason,
            "CountryPP2013" => CountryPP2013,
            "Law2013Country" => Law2013Country,
            "Law2013Okpd2" => Law2013Okpd2,
            "Law2013ViolationReason" => Law2013ViolationReason,
            "MtrGroup" => MtrGroup,
            "MtrReestr" => MtrReestr,
            "MtrReestrName" => MtrReestrName,
            "MtrReestrProducerName" => MtrReestrProducerName,
            "Okpd2PP2013" => Okpd2PP2013,

            _ => {
                return Err(format!(
                    "`{}` является невалидным значением для DictionaryKind",
                    s
                ))
            }
        };

        Ok(val)
    }
}

#[cfg(test)]
mod deser {
    use super::*; //crate::dto::dictionary::{DictionaryList, GetUpdatesResponse, GetUpdatesJsonResponse};

    const TEST_VALUE: &str = r#"{
        "type": "",
        "changed_at": 1747403086676201,
        "entities": [
        {
            "changed_at": 1747403026975937,
            "Unit": [
                {
                    "uuid": "4D54ADC5F55211EC85A5566FF2F30017",
                    "id": 113,
                    "code": "M3",
                    "okei": 113,
                    "text": "Кубический метр",
                    "text_short": "",
                    "is_removed": false,
                    "created_at": 1656249437000000,
                    "created_by": 0,
                    "changed_at": 1656249437000000,
                    "changed_by": 0
                },
                {
                    "uuid": "4D54AD68F55211EC859F566FF2F30017",
                    "id": 920,
                    "code": "LSP",
                    "okei": 920,
                    "text": "Лист печатный",
                    "text_short": "",
                    "is_removed": false,
                    "created_at": 1656249437000000,
                    "created_by": 0,
                    "changed_at": 1656249437000000,
                    "changed_by": 0
                }
            ]
        },
        {
            "changed_at": 1747403026975937,
            "Customer": [
                {
                    "uuid": "920A40DBD22211ED8047005056BD2C78",
                    "id": 20,
                    "nsi_code": "0",
                    "inn": "",
                    "kpp": "",
                    "ogrn": "",
                    "iko": "",
                    "sap_id": 50002556,
                    "purchasing_policy_id": 3625,
                    "kind_id": 0,
                    "budget_item_group_id": 6,
                    "is_ius_p": false,
                    "is_1352": false,
                    "is_not_in_asbu": false,
                    "text": "ООО \"Газпром трансгаз Чайковский\"",
                    "text_short": "ГТЧайковский",
                    "okato_id": 0,
                    "legal_address": "",
                    "created_at": 1680528143000000,
                    "created_by": 0,
                    "changed_at": 1680528143000000,
                    "changed_by": 0
                }
            ]
        },
        {
            "changed_at": 1747403026975937,
            "Currency": [
                {
                    "uuid": "4D601E8EF55211EC870C566FF2F30017",
                    "id": 152,
                    "code": "CLP",
                    "okv": 152,
                    "text": "Песо Чили",
                    "text_short": "Песо",
                    "is_removed": false,
                    "created_at": 1656249437000000,
                    "created_by": 0,
                    "changed_at": 1656249437000000,
                    "changed_by": 0
                }
            ]
        },
        {
            "changed_at": 1747403026975937,
            "PurchasingTrend": [
                {
                    "uuid": "5D601E8EF55211EC870C566FF2F30017",
                    "id": 1,
                    "text": "Закупка услуг",
                    "is_removed": false,
                    "created_at": 1656249437000000,
                    "created_by": 0,
                    "changed_at": 1656249437000000,
                    "changed_by": 0
                }
            ]
        },
        {
            "changed_at": 1747403026975937,
            "SomethingNewAndUnknown": [
                {
                    "uuid": "4D601E8EF55211EC870C566FF2F30017",
                    "id": 152,
                    "chaos": "noah"
                }
            ]
        }
        ]
    }
    "#;

    macro_rules! has_dictionary {
        (@json $res: expr, $variant: ident) => {
            assert!($res.entities.iter().any(|e| {
                if let CommonDictionaryKind::$variant = &e.kind {
                    !e.items.is_empty()
                } else {
                    false
                }
            }))
        };
        ($res: expr, $variant: ident) => {
            assert!($res.entities.iter().any(|e| {
                if let CommonDictionaryList::$variant(records) = &e.items {
                    !records.is_empty()
                } else {
                    false
                }
            }))
        };
    }

    #[test]
    fn get_updates() {
        let res = serde_json::from_str::<GetUpdatesResponse>(TEST_VALUE).unwrap();

        has_dictionary!(res, Customer);
        has_dictionary!(res, Currency);
        has_dictionary!(res, Unit);
        has_dictionary!(res, PurchasingTrend);
    }

    #[test]
    fn get_updates_json() {
        let res =
            serde_json::from_str::<GetUpdatesJsonResponse<serde_json::Value>>(
                TEST_VALUE,
            )
            .expect("ok");

        has_dictionary!(@json res, Customer);
        has_dictionary!(@json res, Currency);
        has_dictionary!(@json res, Unit);
        has_dictionary!(@json res, PurchasingTrend);
    }
}
