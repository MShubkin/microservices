use crate::common::Result;

use super::{constants::*, UpdateAgendaError};
use asez2_shared_db::db_item::{AdaptorableIter, Select};
use asez2_shared_db::{DbAdaptor, Value};
use itertools::Itertools;
use shared_essential::{
    domain::*,
    presentation::dto::processing::{MergedAgendaItem, UpdateAgendaRes},
};

use ahash::AHashMap;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct SimpleAgendaDetails {
    pub(super) agenda: EcAgenda,
    pub(super) plans: Vec<PlanOrAmendment>,
    pub(super) items: Vec<EcAgendaItem>,
    pub(super) partners: Vec<EcPartner>,
    pub(super) attachments: Vec<Attachment>,
}

impl SimpleAgendaDetails {
    pub(super) async fn get_extras(&mut self, pool: &PgPool) -> Result<()> {
        let uuids = self.items.iter().map(|x| Value::from(x.source_uuid));

        let plan_select =
            Select::with_fields(PLAN_FIELDS).in_any(Plan::uuid, uuids);

        self.plans = PlanOrAmendment::select(&plan_select, pool).await?;

        Ok(())
    }

    pub(super) fn into_response(self) -> Result<UpdateAgendaRes> {
        let attachment_list =
            self.attachments.into_iter().adaptors().collect::<Vec<_>>();

        let from_item =
            PlanOrAmendmentRep::from_item_with_fields(RETURN_PLAN_FIELDS);
        let plan_checker = self
            .plans
            .into_iter()
            .map(|x| {
                let uuid = *x.uuid();
                let plan = from_item(x);
                (uuid, plan)
            })
            .collect::<AHashMap<Uuid, PlanOrAmendmentRep>>();

        let (mut items, mut d647_items) = (Vec::new(), Vec::new());
        for item in self
            .items
            .into_iter()
            .filter(|i| !i.is_removed)
            .sorted_by(|a, b| a.number.cmp(&b.number))
        {
            let is_registered_by_d647 = item.is_registered_by_d647;

            let plan = plan_checker
                .get(&item.source_uuid)
                .cloned()
                .ok_or(UpdateAgendaError::NoSource(item.number))?;
            let agenda_item = EcAgendaItemRep::from_item(item, Some(ITEM_FIELDS));
            let merged_item = MergedAgendaItem { agenda_item, plan };

            if is_registered_by_d647 {
                d647_items.push(merged_item)
            } else {
                items.push(merged_item)
            }
        }

        let partner_list = self
            .partners
            .into_iter()
            .filter(|p| !p.is_removed)
            .sorted_by(|a, b| a.role_id.cmp(&b.role_id))
            .adaptors_with_fields(PARTNER_FIELDS)
            .collect::<Vec<_>>();

        Ok(UpdateAgendaRes {
            agenda: EcAgendaRep::from_item::<&str>(
                self.agenda,
                Some(AGENDA_FIELDS),
            ),
            items,
            d647_items,
            partner_list,
            attachment_list,
        })
    }
}
