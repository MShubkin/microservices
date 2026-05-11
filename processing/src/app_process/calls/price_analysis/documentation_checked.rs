use asez2_shared_db::db_item::{AsezTimestamp, Select, SelectionKind};
use shared_essential::{
    domain::{Plan, PlanOrAmendment, PlanOrAmendmentRep, Section},
    presentation::dto::{
        processing::{
            price_analysis::{
                DocumentationCheckedReq, DocumentationCheckedResponseData,
            },
            PlansRequest,
        },
        response_request::{ApiResponse, BusinessMessage, Message, Messages},
    },
};
use sqlx::PgPool;

use crate::{
    app_process::{records::PlanCollectedUpdate, sections::process_sections},
    common::{ProcessingCtx, Result},
};

const DOCUMENTATION_CHECKED_TAG: &str = "/pricing/v1/action/documentation_checked/";
const DOCUMENTATION_CHECKED_FIELDS: &[&str] = &[
    Plan::id,
    Plan::is_check_documentation,
    Plan::check_documentation_date,
    Plan::pricing_expert_id,
];

pub(crate) enum DocumentationCheckedMessage {
    MissingField(&'static str),
    Success,
}

pub(crate) async fn pa_documentation_checked(
    req: DocumentationCheckedReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<DocumentationCheckedResponseData, ()>> {
    tracing::info!(
        kind = "update",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = DOCUMENTATION_CHECKED_TAG,
        req = req,
    );

    let plans = fetch_plans(&req, &proc_ctx.db_pool).await?;
    let mut messages = Messages::default();

    check_plans(&plans, &mut messages);
    if messages.is_error() {
        return Ok(ApiResponse::default().with_messages(messages));
    }

    let updated_plans = update_plans(req, plans, &mut messages, &proc_ctx).await?;

    DocumentationCheckedMessage::Success
        .checked_append(&mut messages, &updated_plans);

    let data = updated_plans
        .into_iter()
        .map(PlanOrAmendmentRep::from_item_with_fields(
            DOCUMENTATION_CHECKED_FIELDS,
        ))
        .collect::<Vec<_>>();

    Ok((data, messages).into())
}

async fn update_plans(
    req: DocumentationCheckedReq,
    mut plans: Vec<PlanOrAmendment>,
    messages: &mut Messages,
    proc_ctx: &ProcessingCtx,
) -> Result<Vec<PlanOrAmendment>> {
    let DocumentationCheckedReq { user_id, .. } = req;

    let now = AsezTimestamp::now();
    plans.iter_mut().for_each(|p| {
        *p.is_check_documentation_mut() = true;
        *p.check_documentation_date_mut() = Some(now);

        *p.changed_at_mut() = now;
        *p.changed_by_mut() = req.user_id;
    });

    let mut recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;

    let updated_plans = PlanOrAmendment::update(
        plans,
        &[Plan::is_check_documentation, Plan::check_documentation_date],
        messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;
    recorder.commit().await?;

    Ok(updated_plans)
}

async fn fetch_plans(
    req: &DocumentationCheckedReq,
    db_pool: &PgPool,
) -> Result<Vec<PlanOrAmendment>> {
    let DocumentationCheckedReq { item_list, .. } = req;

    let plans_req = PlansRequest {
        section: Section::PriceAnalysisPrimaryExpertControl,
        select: Select::with_fields(DOCUMENTATION_CHECKED_FIELDS)
            .add_expand_filter(
                Plan::uuid,
                SelectionKind::In,
                item_list.iter().map(|i| i.uuid),
            ),
        // Не имеет значения для данной секции
        user_id: req.user_id,
    };
    let plans = process_sections(plans_req, db_pool).await?.pure_plans::<Vec<_>>();
    super::check_plans_selection(&plans, item_list)?;
    Ok(plans)
}

fn check_plans(plans: &[PlanOrAmendment], messages: &mut Messages) {
    for poa in plans {
        if poa.pricing_expert_id().is_none() {
            DocumentationCheckedMessage::MissingField("Эксперт АЦ")
                .checked_append(messages, &[poa]);
        }
    }
}

impl BusinessMessage for DocumentationCheckedMessage {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        match self {
            Self::MissingField(field) => Message::error(format!(
                "В ППЗ/ДС {} не заполнено поле {field}.",
                entity.id()
            )),
            Self::Success => Message::success(format!(
                "По ППЗ/ДС {} документация проверена",
                entity.id()
            )),
        }
        .with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        match self {
            Self::MissingField(field) => Message::error(format!(
                "В {} ППЗ/ДС не заполнено поле {field}.",
                entities.len()
            )),
            Self::Success => Message::success(format!(
                "По {} ППЗ/ДС документация проверена",
                entities.len()
            )),
        }
        .with_param_items(entities)
    }
}
