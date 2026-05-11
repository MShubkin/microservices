use env_setup::{MDSCfg, MonolithCfg};
use reqwest::Url;

/// Конфигурация для данных получаемых запросом `/<dictionary>/search_by_id`

/// Источник обновления для заданного поля
#[derive(Debug)]
pub struct IdLookupSource {
    /// Field for replace ("customer_id" or "supplier_id", etc...)
    pub replace_field: String,
    /// Path-part of URL to fetch data
    pub path: String,
}

/// Хост для обновления источников
#[derive(Debug)]
pub struct IdLookupSourceHost {
    pub host: Url,
    pub replacements: Vec<IdLookupSource>,
    pub is_int: bool,
}

/// Полная конфигурация запросов данных из справочников
#[derive(Debug)]
pub struct IdLookupCfg {
    pub records: Vec<IdLookupSourceHost>,
}

impl Default for IdLookupCfg {
    fn default() -> Self {
        let monolith = MonolithCfg::from_env().unwrap().url;
        let master_data = MDSCfg::from_env().unwrap().url;
        IdLookupCfg {
            records: vec![
                // Монолит
                IdLookupSourceHost {
                    host: monolith.clone(),
                    replacements: vec![IdLookupSource {
                        replace_field: "pricing_expert_id".to_string(),
                        path: "/api/json/users/search_by_id/".to_string(),
                    }],
                    is_int: false,
                },
                IdLookupSourceHost {
                    host: monolith.clone(),
                    replacements: vec![IdLookupSource {
                        replace_field: "created_by".to_string(),
                        path: "/api/json/users/search_by_id/".to_string(),
                    }],
                    is_int: false,
                },
                IdLookupSourceHost {
                    host: monolith.clone(),
                    replacements: vec![IdLookupSource {
                        replace_field: "changed_by".to_string(),
                        path: "/api/json/users/search_by_id/".to_string(),
                    }],
                    is_int: false,
                },
                IdLookupSourceHost {
                    host: monolith.clone(),
                    replacements: vec![IdLookupSource {
                        replace_field: "supplier_id".to_string(),
                        path: "/api/json/organization/search_by_id/".to_string(),
                    }],
                    is_int: false,
                },
                IdLookupSourceHost {
                    host: monolith.clone(),
                    replacements: vec![IdLookupSource {
                        replace_field: "user_id".to_string(),
                        path: "/api/json/users/search_by_id/".to_string(),
                    }],
                    is_int: false,
                },
                // НСИ (Master Data Service) - see also aliases in `struct SimpleExtData`
                IdLookupSourceHost {
                    host: master_data.clone(),
                    replacements: vec![IdLookupSource {
                        replace_field: "agenda_status_id".to_string(),
                        path: "/v1/agenda_status/search_by_id/".to_string(),
                    }],
                    is_int: true,
                },
                IdLookupSourceHost {
                    host: master_data.clone(),
                    replacements: vec![IdLookupSource {
                        replace_field: "expert_conclusion_id".to_string(),
                        path: "/v1/expert_conclusion_type/search_by_id/"
                            .to_string(),
                    }],
                    is_int: true,
                },
                IdLookupSourceHost {
                    host: master_data.clone(),
                    replacements: vec![IdLookupSource {
                        replace_field: "pricing_method_id".to_string(),
                        path: "/v1/price_analysis_method/search_by_id/".to_string(),
                    }],
                    is_int: true,
                },
                IdLookupSourceHost {
                    host: master_data,
                    replacements: vec![IdLookupSource {
                        replace_field: "pricing_organization_unit_id".to_string(),
                        path: "/v1/pricing_unit/search_by_id/".to_string(),
                    }],
                    is_int: true,
                },
                IdLookupSourceHost {
                    host: monolith.clone(),
                    replacements: vec![IdLookupSource {
                        replace_field: "okved2_id".to_string(),
                        path: "/api/json/okved2/search_by_id/".to_string(),
                    }],
                    is_int: false,
                },
                IdLookupSourceHost {
                    host: monolith.clone(),
                    replacements: vec![IdLookupSource {
                        replace_field: "okpd2_id".to_string(),
                        path: "/api/json/okpd2/search_by_id/".to_string(),
                    }],
                    is_int: false,
                },
                IdLookupSourceHost {
                    host: monolith,
                    replacements: vec![IdLookupSource {
                        replace_field: "okato_id".to_string(),
                        path: "/api/json/okato/search_by_id/".to_string(),
                    }],
                    is_int: false,
                },
            ],
        }
    }
}

impl IdLookupCfg {
    pub fn fields(&self) -> Vec<String> {
        self.records
            .iter()
            .flat_map(|record| {
                record
                    .replacements
                    .iter()
                    .map(|replacement| replacement.replace_field.clone())
            })
            .collect()
    }
}
