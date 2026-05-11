use super::common::{
    LookupDataFieldMap, LookupDataIdMap, LookupRecordData, LookupRecordId,
};
use crate::domain::{
    CommissionKind, EcAgendaStatus, EcProtocolStatus, ExpertConclusionId,
};
use asez2_shared_db::db_item::EnumDiscriminant;
use std::fmt::Display;

/// Справочные данные, получаемые из db-энумов
#[derive(Debug, Clone)]
pub struct EnumLookupCfg {
    pub data: LookupDataFieldMap,
}

impl Default for EnumLookupCfg {
    fn default() -> Self {
        EnumLookupCfg {
            data: [
                (
                    "agenda_status_id".to_owned(),
                    get_enum_display_values::<EcAgendaStatus>(),
                ),
                (
                    "protocol_status_id".to_owned(),
                    get_enum_display_values::<EcProtocolStatus>(),
                ),
                (
                    "commission_kind_id".to_owned(),
                    get_enum_display_values::<CommissionKind>(),
                ),
                (
                    "expert_conclusion_id".to_owned(),
                    get_enum_display_values::<ExpertConclusionId>(),
                ),
            ]
            .into_iter()
            .collect(),
        }
    }
}
fn get_enum_display_values<T: EnumDiscriminant + Display + Default + 'static>(
) -> LookupDataIdMap {
    T::DISCRIMINANTS
        .iter()
        .map(|(variant, discriminant)| {
            (
                LookupRecordId::with_id(*discriminant as i32),
                LookupRecordData {
                    id: *discriminant as i32,
                    text: variant.to_string(),
                    ..Default::default()
                },
            )
        })
        .collect()
}
