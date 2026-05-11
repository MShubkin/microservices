#![allow(clippy::type_complexity)]
use shared_essential::application::records::Recorder;
use shared_essential::domain::*;
use shared_essential::presentation::dto::response_request::Messages;

use crate::app_process::common::ItemsWithFields;
use crate::common::Result;

use super::constants::{HEADER_FIELDS_TO_UPDATE, ITEM_FIELDS_TO_UPDATE};

pub(super) struct UpdateReq {
    pub(super) protocol: ItemsWithFields<EcProtocol>,
    pub(super) protocol_items: ItemsWithFields<EcProtocolItem>,
    pub(super) estimated_commission_partners: ItemsWithFields<EcPartner>,
    pub(super) attachments: ItemsWithFields<Attachment>,
}

impl UpdateReq {
    /// Внутренняя функция которая осуществляет первичное преобразование сущностей.
    pub(super) fn new(
        protocol: EcProtocol,
        protocol_items: Vec<EcProtocolItem>,
        partners: Vec<EcPartnerRep>,
        attachments: Vec<AttachmentRep>,
    ) -> Result<Self> {
        let protocol = ItemsWithFields::from_items_fields(
            vec![protocol],
            HEADER_FIELDS_TO_UPDATE,
        );
        let protocol_items = ItemsWithFields::from_items_fields(
            protocol_items,
            ITEM_FIELDS_TO_UPDATE,
        );
        let estimated_commission_partners = ItemsWithFields::new(partners)?;
        let attachments = ItemsWithFields::new(attachments)?;

        Ok(Self {
            protocol,
            protocol_items,
            estimated_commission_partners,
            attachments,
        })
    }

    /// Внутренняя функция которая осуществляет первичное преобразование
    /// и обновление сущностей.
    pub(super) async fn update_direct(
        self,
        messages: &mut Messages,
        recorder: &mut Recorder<'_>,
    ) -> Result<(EcProtocol, Vec<EcProtocolItem>)> {
        let protocol = self
            .protocol
            .update_all(messages, recorder)
            .await?
            .pop()
            .expect("single protocol");
        let protocol_items =
            self.protocol_items.upsert_all(messages, recorder).await?;

        self.estimated_commission_partners
            .upsert_all(messages, recorder)
            .await?;
        self.attachments.upsert_all(messages, recorder).await?;
        Ok((protocol, protocol_items))
    }
}
