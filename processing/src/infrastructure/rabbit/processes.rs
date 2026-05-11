//! This submodule contains process adaptors which summarise the `RabbitRunner`
//! processes that each nest runs.
//!
//! The filling for each process exists in the `app_process` module.
use super::FutureAlias;

use crate::common::result::ProcessingError;
use crate::common::{ProcessingCtx, Result};
use crate::presentation::legacy_interaction::LegacyReq;

use broker::{rabbit::RabbitConsumer, BrokerAdapter, Consumer};
use igg_tracing::span_with_fields;
use igg_tracing::tracing_fields::AsezTracingFieldsCollection;
use monolith_service::http::MonolithHttpService;
use rabbit_services::master_data::MasterDataService;
use rabbit_services::properties::{
    basic_properties_to_tracing_fields, AsezRabbitProperties,
};
use rabbit_services::services::processing::ProcessingService;
use rabbit_services::specialized_departments::SpecializedDepartmentsService;
use shared_essential::application::records::RecordCtx;
use shared_essential::presentation::dto::error::AsezError;
use shared_essential::presentation::dto::{processing::*, Source};

use amqprs::channel::BasicPublishArguments;
use amqprs::BasicProperties;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, trace, Instrument};

/// Хендлер процессинга, которые обрабатывает входящие запросы
/// по кролику. Существует для универсальных сигнатур ручек, которые
/// могут принимать любой аргумент, который имплементирует [`FromProcessingRequest`]
trait ProcessingHandler<R, T, S>: Send + Sync {
    fn call(self, req: R, proc_ctx: ProcessingCtx) -> S;
}

trait FromProcessingRequest {
    fn from_processing_context(proc_ctx: ProcessingCtx) -> Self;
}

fn processing_handler<H, R, S, T>(handler: H, req: R, proc_ctx: ProcessingCtx) -> S
where
    H: ProcessingHandler<R, T, S>,
{
    handler.call(req, proc_ctx)
}

macro_rules! impl_handler {
    ($($T:ident),*) => {
        #[allow(unused_parens, non_snake_case, unused_variables)]
        impl<R, S, Fn, $($T),*> ProcessingHandler<R, ($($T),*), S> for Fn
        where
            Fn: FnOnce(R, $($T),*) -> S + Send + Sync,
            R: Send + 'static,
            S: Send,
            $($T: FromProcessingRequest,)*
        {
            fn call(self, req: R, nest: ProcessingCtx) -> S
            {
                $(
                    let $T = $T::from_processing_context(nest.clone());
                )*
                self(
                    req,
                    $($T),*
                )
            }
        }
    }
}

impl_handler!();
impl_handler!(T1);
impl_handler!(T1, T2);
impl_handler!(T1, T2, T3);
impl_handler!(T1, T2, T3, T4);
impl_handler!(T1, T2, T3, T4, T5);

macro_rules! handle {
    ($fun:ident,$c:expr,$rsvp:expr,$nest:expr,$span:expr) => {{
        tokio::task::spawn(
            async move {
                let nest = $nest.clone();

                let r =
                    processing_handler(crate::app_process::$fun, $c, nest.clone())
                        .await;
                let res = publish_result(r, &nest, $rsvp).await;
                let action = stringify!($fun);

                trace_process("broker", &$nest.entrance_queue_name, action, res)
            }
            .instrument($span),
        );
    }};
}

macro_rules! enumerate_processes {
    ($msg:expr,$rsvp:expr,$nest:expr,$span:expr,$req_type:ident, $($variant:ident => $called_function:ident,)+) => {{
        match $msg {
            $(
                 $req_type::$variant(x) => handle!($called_function, x, $rsvp, $nest, $span),
            )+
        }
    }};
}

async fn get_message<T>(
    proc_ctx: &ProcessingCtx,
    consumer: &mut RabbitConsumer,
) -> Result<(T, Option<String>, Option<AsezTracingFieldsCollection>)>
where
    T: std::fmt::Debug + Sync + Send + for<'a> serde::Deserialize<'a>,
{
    // timeout для того чтобы если consumer завис, можно было
    // определить и пересоздать.
    let timeout = std::time::Duration::from_millis(20_000);
    let consumed = consumer.consume_with_timeout(timeout).await;
    // TODO: Do we need a notification or something to be sent
    // if we fail to consume from the queue? If we do nothing
    // then we're "swallowing" the signal here like a black hole.
    trace_process(
        "broker",
        &proc_ctx.entrance_queue_name,
        "consume from broker",
        consumed.map_err(Into::into),
    )
    .and_then(|x| {
        Ok((
            x.content,
            x.properties.reply_to().map(|x| x.to_string()),
            basic_properties_to_tracing_fields(&x.properties)?,
        ))
    })
}

/// This is the default function for processing the plans queue. The internal
/// components are pulled from the processing (business logic) module.
#[tracing::instrument(skip_all)]
pub(crate) fn process_legacy_queue<'a>(
    mut proc_ctx: ProcessingCtx,
    entrance: &'a mut RabbitConsumer,
) -> FutureAlias<'a> {
    Box::pin(async move {
        info!(kind = "legacy_request", "Processing proc_ctx {:?}", proc_ctx);
        // NB: Messages can get lost without a reply here.
        let (msg, rsvp, tracing_fields) = get_message(&proc_ctx, entrance).await?;
        proc_ctx.tracing_fields = tracing_fields.clone();

        let span = span_with_fields!(tracing_fields, "legacy_queue_span");

        info!(kind = "legacy_request", "Запрос от монолита");
        trace!(kind = "legacy_request", body = %msg);
        // Because we get these messages from the external "planning" module
        // which does not always follow our contracts, we examine it more closely.
        let body = serde_json::from_value(msg).map_err(|e| {
            error!(
                kind = "legacy_request",
                "Ошибка обработки запроса с монолита: {}", e
            );
            e
        })?;
        enumerate_processes!(
            body, rsvp, proc_ctx, span,
            LegacyReq,
            InsertUpdateLegacyPlans => upsert_legacy_plan,
            InsertUpdateLegacyAmendments => upsert_legacy_amendment,
        );
        Ok(())
    })
}

/// This is the default function for processing the plans queue. The internal
/// components are pulled from the processing (business logic) module.
#[tracing::instrument(skip_all)]
pub(crate) fn process_plans_queue<'a>(
    mut proc_ctx: ProcessingCtx,
    entrance: &'a mut RabbitConsumer,
) -> FutureAlias<'a> {
    use ProcessingRequest as Req;

    Box::pin(async move {
        info!(kind = "request", "Processing proc_ctx {:?}", proc_ctx);
        // TODO: Do we need a notification or something to be sent
        // if we fail to consume from the queue? If we do nothing
        // then we're "swallowing" the signal here like a black hole.
        let (msg, rsvp, tracing_fields) = get_message(&proc_ctx, entrance).await?;
        proc_ctx.tracing_fields = tracing_fields.clone();

        let span = span_with_fields!(tracing_fields, "plans_queue_span");

        info!(kind = "request", "Запрос от микросервисов");
        trace!(kind = "request", body = ?msg);

        enumerate_processes!(
            msg, rsvp, proc_ctx, span,
            Req,
            // Request from the monolith.
            // NB: For now we add it here.
            InsertUpdateLegacyPlans => upsert_legacy_plan,
            // Request from the monolith.
            // NB: For now we add it here.
            InsertUpdateLegacyAmendments => upsert_legacy_amendment,
            // Here we process from "/rest/v1/plans/get"
            GetPlans => get_plans,
            GetPlansCount => get_plans_count,
            // Here we process from "/rest/v1/plan/get"
            GetCompletePlans => get_complete_plans,
            GetCompleteContractAmendments => get_complete_contract_amendments,
            GetPlanVersion => get_plan_version,
            GetContractAmendmentVersion => get_contract_amendment_version,
            GetAttachmentsMeta => get_attachments_meta,
            // Here we process from "/rest/v1/estimated_commission/pre_request/agenda_create/"
            PreCreateAgenda => pre_create_agenda,
            // Here we process from "/rest/estimated_commission/v1/pre_request/agenda_remove/"
            PreRequestAgendaRemove => pre_request_agenda_remove,
            // Here we process from "/rest/estimated_commission/v1/action/agenda_remove/"
            AgendaRemove => action_agenda_remove,
            PreRemoveAgendaItems => pre_remove_agenda_items,
            // Here we process from "/rest/estimated_commission/v1/action/protocol_agreement/"
            ProtocolAgreement => action_protocol_agreement,
            // Here we process from "/rest/estimated_commission/v1/pre_request/protocol_agreement/"
            PreProtocolAgreement => pre_request_protocol_agreement,
            // Here we process from "/rest/v1/estimated_commission/action/create_agenda"
            CreateAgenda => create_agenda,
            PreCreateProtocol => pre_create_protocol,
            // Here we process from "/rest/v1/estimated_commission/action/create_agenda"
            CreateProtocol => create_protocol,
            // Here we process from "/rest/v1/estimated_commission/update/plan"
            UpdatePlans => update_plans,
            // Процессинг "POST /rest/estimated_commission/v1/pre_request/add_plans_agenda/" эндпоинта
            PreAddPlansAgenda => pre_add_plans_agenda,
            AddPlansAgenda => add_plans_agenda,
            PreTransferPlansAgenda => pre_transfer_plans_agenda,
            TransferPlansAgenda => transfer_plans_agenda,
            // Процессинг "POST /rest/estimated_commission/v1/pre_request/add_plans_protocol/"
            PreAddPlansProtocol => pre_add_plans_protocol,
            AddPlansProtocol => add_plans_protocol,
            GetAgendaDetails => get_agenda_details,
            // Процессинг "/rest/estimated_commission/v1/get/agenda_list" эндпоинта
            GetAgendaList => get_agenda_list,
            // Процессинг "/rest/estimated_commission/v1/get/agenda_list_by_date" эндпоинта
            GetAgendaListByDate => get_agenda_list_by_date,
            GetProtocolDetails => get_protocol_details,
            GetProtocolList => get_protocol_list,
            GetProtocolListByAgenda => get_protocol_list_by_agenda,
            GetProtocolListByDate => get_protocol_list_by_date,
            GetAgendaItemsByIdRange => get_agenda_items_by_id_range,
            GetProtocolItemsByIdRange => get_protocol_items_by_id_range,
            GetAgendaItemsForProtocolCreate => get_agenda_items_for_protocol_create,
            CancelPlan => cancel_plan,
            PreCancelPlan => pre_cancel_plan,
            PreChangeForm => pre_change_form,
            ChangeForm => change_form,
            ReturnToCustomer => return_to_customer,
            PreReturnToCustomer => pre_return_to_customer,
            ReturnToExpert => return_to_expert,
            PreReturnToExpert => pre_return_to_expert,
            GetPartners => get_partners,
            AssignExpert => assign_expert,
            AssignExpertMass => assign_expert_mass,
            RemoveProtocol => remove_protocol,
            PreRemoveProtocol => pre_remove_protocol,
            ApproveProtocol => approve_protocol,
            PreApproveProtocol => pre_approve_protocol,
            ConfirmDecision => confirm_decision,
            GetAgendaItemList => get_item_list,
            PreSendProtocolForSigning => pre_send_protocol_for_signing,
            SendProtocolForSigning => send_protocol_for_signing,
            ChangeCommissionDateReq => change_commission_date,
            PreChangeCommissionDateReq => pre_change_commission_date,
            PreAgendaSend => pre_agenda_send,
            AgendaSend => agenda_send,
            ApprovePlans => action_approve,
            PreApprovePlans => pre_approve,
            PaReturnToCustomer => pa_return_to_customer,
            PaPreReturnToCustomer => pa_pre_return_to_customer,
            PaDocumentationChecked => pa_documentation_checked,
            PaGetSectionsCount => pa_get_sections_count,
            PaCompleteLotting => pa_complete_lotting,
            EcGetSectionsCount => ec_get_sections_count,
            UpdateAgenda => update_agenda,
            UpdateProtocol => update_protocol,
            PaPreRequestDocumentation => pa_pre_request_documentation,
            PaRequestDocumentation => pa_request_documentation,
            PaPreRequestDocumentsForExpert => pa_pre_request_documents_for_expert,
            PriceDetermined => pa_price_determined,
            ApproveByChief => pa_approve_by_chief,
            PaPreDeclineByChief => pa_pre_decline_by_chief,
            PaDeclineByChief => pa_decline_by_chief,
            PaUpdatePlan => pa_update_plan,
            PaUpdateContractAmendment => pa_update_contract_amendment,
            PaReviewProgress => pa_review_progress,
            GetPlansWithLastAgendaItems => get_plans_with_last_agenda_items,
            PricingResult => pa_pricing_result,
            PricingUser => get_price_analysis_user,
            ExportData => export_data,
            ExportSpecification => export_specification,
            ImportSpecification => import_specification,
            ImportItemListSpecific => import_item_list_specific,
            PricingReportCommon => pricing_report_common_data,
            PricingReportSavings => pricing_report_savings_data,
            PricingReportCommission => pricing_report_commission_data,
            GetRetrospective => get_retrospective,
        );
        Ok(())
    })
}

/// Exists for dry.
#[tracing::instrument(skip_all)]
async fn publish_result<C>(
    result: Result<C>,
    n: &ProcessingCtx,
    rsvp: Option<String>,
) -> Result<()>
where
    C: serde::Serialize + Default + Send + Sync,
{
    let result = result.map_err(AsezError::from);
    info!(
        kind = "broker",
        "Publishing to: {} {rsvp:?}, OK?: {}",
        n.entrance_queue_name,
        result.is_ok()
    );
    trace!(
        kind = "broker",
        "Publishing ({rsvp:?}): {}",
        serde_json::to_string_pretty(&result).unwrap()
    );
    // If direct reply mechanism is not set, we spawn the reply into the
    // Plans_Response queue. For now. (TODO)
    let exit_name = rsvp.unwrap_or_default();

    let args = BasicPublishArguments::new("", &exit_name);

    let props = BasicProperties::default()
        .with_content_type("application/json")
        .with_persistence(true)
        .finish();

    let publisher =
        n.adaptor.register_publisher(props, args).await.map_err(|e| {
            error!(
                kind = "broker",
                "Processing: Error registering response publisher on {}: {}",
                n.entrance_queue_name,
                e
            );
            e
        })?;
    let expiration = Duration::from_millis(ProcessingService::DEFAULT_EXPIRATION);

    publisher
        .publish_with_expiration(&result, expiration)
        .await
        .map_err(|e| {
            error!(
                kind = "broker",
                "Processing: Error publishing response on {}: {}",
                n.entrance_queue_name,
                e
            );
            e
        })?;
    Ok(())
}

fn trace_process<T: std::fmt::Debug + Sync>(
    kind: &str,
    queue: &str,
    stage: &str,
    res: Result<T>,
) -> Result<T> {
    match res {
        Ok(ref d) => trace(kind, queue, stage, d),
        Err(ref e) => trace_err(kind, stage, queue, e),
    };
    res
}

fn trace_err(kind: &str, stage: &str, queue: &str, e: &ProcessingError) {
    if e.is_broker_timeout() {
        trace!(
            kind = kind,
            "Processing: Timeout at \"{s}\" stage on queue \"{q}\": {err}",
            s = stage,
            q = queue,
            err = e,
        );
    } else {
        error!(
            kind = kind,
            "Processing: Error at \"{s}\" stage on queue \"{q}\": {err}",
            s = stage,
            q = queue,
            err = e,
        );
    }
}

/// Функция существует для "DRY". Записывает логи на уровне trace.
fn trace(kind: &str, queue: &str, stage: &str, content: &dyn std::fmt::Debug) {
    trace!(
        kind = kind,
        "Processing queue \"{q}\", stage \"{s}\": {content:#?}",
        q = queue,
        s = stage,
        content = content,
    );
}

impl FromProcessingRequest for ProcessingCtx {
    fn from_processing_context(proc_ctx: ProcessingCtx) -> Self {
        proc_ctx
    }
}

impl FromProcessingRequest for Arc<PgPool> {
    fn from_processing_context(proc_ctx: ProcessingCtx) -> Self {
        proc_ctx.db_pool
    }
}

impl FromProcessingRequest for Arc<MonolithHttpService> {
    fn from_processing_context(proc_ctx: ProcessingCtx) -> Self {
        proc_ctx.monolith_service
    }
}

impl FromProcessingRequest for RecordCtx {
    fn from_processing_context(proc_ctx: ProcessingCtx) -> Self {
        proc_ctx.create_record_context()
    }
}

impl FromProcessingRequest for SpecializedDepartmentsService {
    fn from_processing_context(proc_ctx: ProcessingCtx) -> Self {
        let ProcessingCtx {
            adaptor,
            tracing_fields,
            ..
        } = proc_ctx;
        let mut rabbit_properties = AsezRabbitProperties::default();
        if let Some(fields) = tracing_fields {
            rabbit_properties.add_tracing_fields(&fields);
        }
        SpecializedDepartmentsService::new(
            adaptor,
            rabbit_properties,
            Source::Processing,
        )
    }
}

impl FromProcessingRequest for MasterDataService {
    fn from_processing_context(proc_ctx: ProcessingCtx) -> Self {
        let mut rabbit_properties = AsezRabbitProperties::default();
        if let Some(fields) = &proc_ctx.tracing_fields {
            rabbit_properties.add_tracing_fields(fields);
        }

        MasterDataService::new(
            proc_ctx.adaptor.clone(),
            rabbit_properties,
            Source::Processing,
        )
    }
}
