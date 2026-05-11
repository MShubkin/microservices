use asez2_tables::master_data::routes::RouteHeader;

use super::rules_lawyer::*;

use super::*;

use crate::domain::tables::*;

/// Дефолтная имплементация [`ProcessUpsert`]
macro_rules! impl_puh {
    ($name:ident, $status_field:expr) => {
        impl ProcessUpsert for $name {
            const CTX_UPDATE_FIELDS: &'static [&'static str] =
                &["changed_at", "changed_by"];
            const STATUS_FIELD: Option<&'static str> = $status_field;

            fn generate_uuid_if_needed(&mut self) {
                if self.uuid == Uuid::nil() {
                    self.uuid = uuid::Uuid::new_v4();
                }
            }

            fn apply_update_ctx(&mut self, ctx: &UpdateCtx) {
                self.changed_at = ctx.timestamp;
                self.changed_by = ctx.user_id;
            }

            fn apply_insert_ctx(&mut self, ctx: &UpdateCtx) {
                self.changed_at = ctx.timestamp;
                self.changed_by = ctx.user_id;
                self.created_at = ctx.timestamp;
                self.created_by = ctx.user_id;
            }
        }
    };

    ($name:ident, $status_field:expr, keep_context) => {
        impl ProcessUpsert for $name {
            const CTX_UPDATE_FIELDS: &'static [&'static str] =
                &["changed_at", "changed_by"];
            const STATUS_FIELD: Option<&'static str> = $status_field;

            fn generate_uuid_if_needed(&mut self) {
                if self.uuid == Uuid::nil() {
                    self.uuid = uuid::Uuid::new_v4();
                }
            }

            fn apply_update_ctx(&mut self, ctx: &UpdateCtx) {
                if !ctx.is_external {
                    self.changed_at = ctx.timestamp;
                    self.changed_by = ctx.user_id;
                }
            }

            fn apply_insert_ctx(&mut self, ctx: &UpdateCtx) {
                if !ctx.is_external {
                    self.changed_at = ctx.timestamp;
                    self.changed_by = ctx.user_id;
                    self.created_at = ctx.timestamp;
                    self.created_by = ctx.user_id;
                }
            }
        }
    };
}

impl_puh!(EcProtocolItem, None);
impl_puh!(EcPartner, None);
impl_puh!(Attachment, None);
impl_puh!(EcAgendaItem, None);
impl_puh!(EcProtocol, Some(EcProtocol::status_id));
impl_puh!(EcAgenda, Some(EcAgenda::status_id));
impl_puh!(RouteHeader, None);

impl_puh!(Plan, Some(Plan::status_id), keep_context);
impl_puh!(ContractAmendment, Some(ContractAmendment::status_id), keep_context);
impl_puh!(PlanItemFull, None, keep_context);
impl_puh!(ContractAmendmentItem, None, keep_context);

impl ProcessUpsert for DocumentApprover {
    const CTX_UPDATE_FIELDS: &'static [&'static str] =
        &["changed_at", "changed_by"];

    const STATUS_FIELD: Option<&'static str> = None;

    fn generate_uuid_if_needed(&mut self) {
        if self.uuid == Uuid::nil() {
            self.uuid = uuid::Uuid::new_v4();
        }
    }

    fn apply_update_ctx(&mut self, ctx: &UpdateCtx) {
        self.changed_at = ctx.timestamp;
        self.changed_by = ctx.user_id;
    }

    fn apply_insert_ctx(&mut self, ctx: &UpdateCtx) {
        if !ctx.is_external {
            self.changed_at = ctx.timestamp;
            self.changed_by = ctx.user_id;
            self.created_at = ctx.timestamp;
            self.created_by = ctx.user_id;
        } else {
            self.created_at = ctx.timestamp; // Указать текущую дату и время создания записи в БД
                                             // self.created_by Заполняется из запроса структуры specialized_departments - created_by
            self.changed_by = self.created_by; // указывается значение равное created_by
            self.changed_at = self.created_at; // указывается значение равное created_at
        }
    }
}

impl RulesLawyer for Plan {}

impl RulesLawyer for ContractAmendment {}
