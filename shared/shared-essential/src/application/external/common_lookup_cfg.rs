use super::monolith::DictionaryKind;
use ahash::AHashMap;

/// Конфигурация для данных получаемых из монолита запросом `/master_data/get_updates/0/`
#[derive(Debug)]
pub struct CommonLookupReplacement {
    /// Field for replace ("country_id", "currency_id", "vat_id" etc...)
    pub field_name: String,
    /// `DictionaryKind` - source of replacement data (preloaded from `/master_data/get_updates/0/`)
    /// see `monolith.rs` for details
    pub dictionary_kind: DictionaryKind,
}

#[derive(Debug)]
pub struct CommonLookupCfg {
    pub records: Vec<CommonLookupReplacement>,
}

impl Default for CommonLookupCfg {
    fn default() -> Self {
        CommonLookupCfg {
            records: vec![
                CommonLookupReplacement {
                    field_name: "category_id".to_string(),
                    dictionary_kind: DictionaryKind::Category,
                },
                CommonLookupReplacement {
                    field_name: "country_id".to_string(),
                    dictionary_kind: DictionaryKind::Country,
                },
                CommonLookupReplacement {
                    field_name: "currency_id".to_string(),
                    dictionary_kind: DictionaryKind::Currency,
                },
                CommonLookupReplacement {
                    field_name: "customer_id".to_string(),
                    dictionary_kind: DictionaryKind::Customer,
                },
                CommonLookupReplacement {
                    field_name: "department_id".to_string(),
                    dictionary_kind: DictionaryKind::Department,
                },
                CommonLookupReplacement {
                    field_name: "purchasing_method_id".to_string(),
                    dictionary_kind: DictionaryKind::PurchasingMethod,
                },
                CommonLookupReplacement {
                    field_name: "section_id".to_string(),
                    dictionary_kind: DictionaryKind::Section,
                },
                CommonLookupReplacement {
                    field_name: "unit_id".to_string(),
                    dictionary_kind: DictionaryKind::Unit,
                },
                CommonLookupReplacement {
                    field_name: "vat_id".to_string(),
                    dictionary_kind: DictionaryKind::Vat,
                },
                CommonLookupReplacement {
                    field_name: "single_supplier_reason_id".to_string(),
                    dictionary_kind: DictionaryKind::PurchasingPolicyItem,
                },
                CommonLookupReplacement {
                    field_name: "contract_amendment_types".to_string(),
                    dictionary_kind: DictionaryKind::ContractAmendmentKind,
                },
            ],
        }
    }
}

impl CommonLookupCfg {
    pub fn as_map(&self) -> AHashMap<String, DictionaryKind> {
        AHashMap::from_iter(
            self.records
                .iter()
                .map(|record| (record.field_name.clone(), record.dictionary_kind)),
        )
    }
}
