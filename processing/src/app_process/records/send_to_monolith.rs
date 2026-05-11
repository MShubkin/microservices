use crate::common::NO_SEND_TO_PLANNING;
use crate::common::{monolith_sender, MonolithSenderObject, Result};

use asez2_shared_db::{DbItem, Value};
use itertools::Itertools;
use shared_essential::application::records::Recorder;
use shared_essential::domain::*;

pub(crate) async fn send_plans_to_monolith(
    plans: &[Plan],
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    // Если обновляем статус ид и есть команда посылать на монолит,
    // то берём все ДС со ВСЕМИ позициями и стреляем.
    if std::env::var(NO_SEND_TO_PLANNING).is_ok() {
        return Ok(());
    }

    if let Some(plans) = monolith_sender::plans_for_monolith(
        plans.iter().map(|x| Value::from(x.uuid)),
        recorder,
    )
    .await?
    {
        MonolithSenderObject::new(plans).insert(recorder.tx()).await?;
    }
    Ok(())
}

pub(crate) async fn send_to_monolith(
    items: &[PlanOrAmendment],
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    // Если обновляем статус ид и есть команда посылать на монолит,
    // то берём все ДС со ВСЕМИ позициями и стреляем.
    if std::env::var(NO_SEND_TO_PLANNING).is_ok() {
        return Ok(());
    }

    let (plans, amendments): (Vec<_>, Vec<_>) =
        items.iter().partition_map(|x| match x {
            PlanOrAmendment::Plan(x) => {
                itertools::Either::Left(Value::from(x.uuid))
            }
            PlanOrAmendment::Amendment(x) => {
                itertools::Either::Right(Value::from(x.uuid))
            }
        });

    if let Some(plans) =
        monolith_sender::plans_for_monolith(plans.into_iter(), recorder).await?
    {
        MonolithSenderObject::new(plans).insert(recorder.tx()).await?;
    }
    if let Some(amendments) =
        monolith_sender::amendments_for_monolith(amendments.into_iter(), recorder)
            .await?
    {
        MonolithSenderObject::new(amendments).insert(recorder.tx()).await?;
    }
    Ok(())
}
