#![allow(clippy::type_complexity)]
use ahash::AHashSet;
use shared_essential::application::records::Recorder;
use shared_essential::{domain::*, presentation::dto::response_request::*};
use uuid::Uuid;

use super::constants::{AGENDA_UPDATE_FIELDS, AGENDA_UPDATE_ITEM_FIELDS};
use super::simple_details::SimpleAgendaDetails;
use super::Result;
use crate::app_process::common::ItemsWithFields;

#[derive(Debug)]
pub(super) struct UpdateReq {
    pub(super) agenda: ItemsWithFields<EcAgenda>,
    pub(super) items: ItemsWithFields<EcAgendaItem>,
    pub(super) partners: ItemsWithFields<EcPartner>,
    pub(super) attachments: ItemsWithFields<Attachment>,
    pub(super) return_items: AHashSet<Uuid>,
}

fn from_agenda_header(header: EcAgenda) -> ItemsWithFields<EcAgenda> {
    ItemsWithFields::from_items_fields(
        std::iter::once(header),
        AGENDA_UPDATE_FIELDS,
    )
}

fn from_agenda_items(items: Vec<EcAgendaItem>) -> ItemsWithFields<EcAgendaItem> {
    ItemsWithFields::from_items_fields(items, AGENDA_UPDATE_ITEM_FIELDS)
}

impl UpdateReq {
    /// Внутренняя функция которая осуществляет первичное преобразование сущностей.
    pub(super) fn new(
        header: EcAgenda,
        items: Vec<EcAgendaItem>,
        partners: Vec<EcPartnerRep>,
        attachments: Vec<AttachmentRep>,
    ) -> Result<Self> {
        let agenda = from_agenda_header(header);
        let items = from_agenda_items(items);
        let partners = ItemsWithFields::new(partners)?;
        let attachments = ItemsWithFields::new(attachments)?;

        let return_items = items.items.iter().map(|i| i.uuid).collect();

        Ok(Self {
            agenda,
            items,
            partners,
            attachments,
            return_items,
        })
    }

    /// Внутренняя функция которая осуществляет первичное преобразование
    /// и обновление сущностей.
    pub(super) async fn update_direct(
        self,
        msg: &mut Messages,
        recorder: &mut Recorder<'_>,
    ) -> Result<SimpleAgendaDetails> {
        let agenda = self
            .agenda
            .update_all(msg, recorder)
            .await?
            .pop()
            .expect("should be single item");
        let items = self.items.upsert_all(msg, recorder).await?;
        let partners = self.partners.upsert_all(msg, recorder).await?;
        let attachments = self.attachments.upsert_all(msg, recorder).await?;

        let items = items
            .into_iter()
            .filter(|i| self.return_items.contains(&i.uuid))
            .collect();

        Ok(SimpleAgendaDetails {
            agenda,
            items,
            plans: vec![],
            partners,
            attachments,
        })
    }
}
