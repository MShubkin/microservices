use crate::domain::{
    ContractAmendment, ContractAmendmentItem, ContractAmendmentItemLegacyRep,
    ContractAmendmentItemRep, ContractAmendmentLegacyRep, ContractAmendmentRep,
    DocumentApproverRep, Plan, PlanItemFull, PlanItemFullRep, PlanItemLegacyRep,
    PlanLegacyRep, PlanRep, PlanRetrospectiveLegacy, PlanRetrospectiveRep,
    PlanningDocumentApprover,
};

use asez2_shared_db::result::SharedDbError;
use serde::{Deserialize, Serialize};

// /// Тип объекта который обновляется до SRM.
// #[derive(Debug, Serialize)]
// pub enum SrmObjectType {
//     Plan,
//     PlanItem,
//     Amendment,
//     AmendmentItem,
// }

// #[derive(Debug, Serialize)]
// pub struct SrmUpdateItems<T> {
//     pub parent_uuid: Option<Uuid>,
//     pub rows: Vec<T>,
// }

// #[derive(Debug, Serialize)]
// pub struct SrmUpdate<T> {
//     pub object_type: SrmObjectType,
//     pub items: Vec<SrmUpdateItems<T>>,
// }

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct DataFromSrm<T, U> {
    pub header: T,
    pub items: Vec<U>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct DataFromSrmExt<T, U, V> {
    /// Данные ППЗ.
    #[serde(flatten)]
    pub data: DataFromSrm<T, U>,
    #[serde(flatten)]
    pub ext: V,
}

impl<T, U, V> DataFromSrmExt<T, U, V> {
    pub fn split(self) -> (T, Vec<U>, V) {
        (self.data.header, self.data.items, self.ext)
    }
    pub fn split_into<W>(self) -> (T, Vec<U>, W)
    where
        V: Into<W>,
    {
        (self.data.header, self.data.items, self.ext.into())
    }
}

pub type PlanFromSrmExt<T> = DataFromSrmExt<PlanLegacyRep, PlanItemLegacyRep, T>;
pub type ContractAmendmentFromSrmExt<T> =
    DataFromSrmExt<ContractAmendmentLegacyRep, ContractAmendmentItemLegacyRep, T>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
#[serde(tag = "request_kind", content = "request_data")]
pub enum PlansAmendmentsFromSrmExt<T> {
    #[serde(rename = "InsertUpdateLegacyPlans")]
    Plans(Vec<PlanFromSrmExt<T>>),
    #[serde(rename = "InsertUpdateLegacyAmendments")]
    Amendments(Vec<ContractAmendmentFromSrmExt<T>>),
}

/// ППЗ из монолита
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct PlanFromSrm {
    /// Заголовок
    pub header: PlanLegacyRep,
    /// Позиции
    pub items: Vec<PlanItemLegacyRep>,
    /// Ретроспектива
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrospective_list: Option<Vec<PlanRetrospectiveLegacy>>,
    /// Профильные департаменты.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specialized_departments: Option<Vec<PlanningDocumentApprover>>,
}

pub type InsertUpdateSrmPlansReq = Vec<PlanFromSrm>;

pub struct PlanWithFullItems {
    pub plan: Plan,
    pub items: Vec<PlanItemFull>,
}

pub struct PlanFromSrmParts {
    pub header: PlanRep,
    pub items: Vec<PlanItemFullRep>,
    pub retrospective_list: Vec<PlanRetrospectiveRep>,
    pub specialized_departments: Vec<DocumentApproverRep>,
}

impl PlanFromSrm {
    pub fn try_to_parts_rep(self) -> Result<PlanFromSrmParts, SharedDbError> {
        let header: PlanRep =
            self.header.try_into().map_err(SharedDbError::Other)?;
        let items = self
            .items
            .into_iter()
            .map(PlanItemFullRep::from)
            .map(Into::into)
            .collect::<Vec<_>>();
        let retrospective_list = self
            .retrospective_list
            .into_iter()
            .flatten()
            .map(|r| r.to_plan_retrospective_rep(header.id, header.uuid))
            .collect::<Vec<_>>();
        let specialized_departments = self
            .specialized_departments
            .into_iter()
            .flatten()
            .map(|x| x.to_document_approver_rep(header.id, header.uuid))
            .collect();

        Ok(PlanFromSrmParts {
            header,
            items,
            retrospective_list,
            specialized_departments,
        })
    }
}

/// ППЗ из монолита
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct AmendmentFromSrm {
    /// Заголовок
    pub header: ContractAmendmentLegacyRep,
    /// Позиции
    pub items: Vec<ContractAmendmentItemLegacyRep>,
    /// Ретроспектива
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrospective_list: Option<Vec<PlanRetrospectiveLegacy>>,
    /// Профильные департаменты.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specialized_departments: Option<Vec<PlanningDocumentApprover>>,
}
pub type InsertUpdateSrmAmendmentsReq = Vec<AmendmentFromSrm>;

pub struct AmendmentWithFullItems {
    pub amendment: ContractAmendment,
    pub items: Vec<ContractAmendmentItem>,
}

pub struct AmendmentFromSrmParts {
    pub header: ContractAmendmentRep,
    pub items: Vec<ContractAmendmentItemRep>,
    pub retrospective_list: Vec<PlanRetrospectiveRep>,
    pub specialized_departments: Vec<DocumentApproverRep>,
}

impl AmendmentFromSrm {
    pub fn try_to_part_rep(self) -> Result<AmendmentFromSrmParts, SharedDbError> {
        let header: ContractAmendmentRep =
            self.header.try_into().map_err(SharedDbError::Other)?;
        let items = self
            .items
            .into_iter()
            .map(ContractAmendmentItemRep::from)
            .map(Into::into)
            .collect::<Vec<_>>();
        let retrospective_list = self
            .retrospective_list
            .into_iter()
            .flatten()
            .map(|r| r.to_plan_retrospective_rep(header.id, header.uuid))
            .collect::<Vec<_>>();
        let specialized_departments = self
            .specialized_departments
            .into_iter()
            .flatten()
            .map(|x| x.to_document_approver_rep(header.id, header.uuid))
            .collect();

        Ok(AmendmentFromSrmParts {
            header,
            items,
            retrospective_list,
            specialized_departments,
        })
    }
}
