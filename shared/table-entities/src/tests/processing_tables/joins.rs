//! Tests for joins of existing structures are carried out here.
//! This is because they cannot be carried out in shared-db in a sane manner.
use crate::test_setup::run_db_test;

use asez2_shared_db::db_item::{joined::JoinTo, selection::FieldSortKind, Select};
use tokio::test;

#[test]
#[ignore = "Тест по какой-то причине не работает и нуждается в исправлении в следующем МР"]
async fn test_joined_plan_header_esa_item_esp_item() {
    use crate::processing::plan::{
        JoinedPlanEcAgendaItemEcProtocolItem as JoinedPlan,
        JoinedPlanEcAgendaItemEcProtocolItemSelector as JoinedPlanSel, Plan,
    };
    use crate::{EcAgendaItem, EcProtocolItem};
    // Join is derived from:
    // ```
    // impl_join_on!(Plan:uuid => EcAgendaItem:plan_uuid);
    // impl_join_on!(Plan:uuid => EcProtocolItem:plan_uuid);
    // joined!(
    //     plan: Plan,
    //     agenda_item: EcAgendaItem[Plan => EcAgendaItem],
    //     protocol_item: EcProtocolItem[Plan => EcProtocolItem],
    // );
    // ```

    run_db_test(|pool| async move {
        {
            // Test without distinct
            let select = Select::full::<Plan>();
            let select_ai = Select::full::<EcAgendaItem>()
                .add_replace_order("changed_at", FieldSortKind::Desc);
            let select_pi = Select::full::<EcProtocolItem>()
                .add_replace_order("changed_at", FieldSortKind::Desc);

            let h: Vec<JoinedPlan> = JoinedPlanSel::new(select)
                .set_agenda_item(EcAgendaItem::join_default().selecting(select_ai))
                .set_protocol_item(
                    EcProtocolItem::join_default().selecting(select_pi),
                )
                .get(&*pool)
                .await
                .unwrap();

            let h2 = h
                .iter()
                .map(|x| {
                    (x.plan.uuid, &x.agenda_item.number, &x.protocol_item.number)
                })
                .collect::<Vec<_>>();
            println!("{:#?}", h2);
            assert_eq!(h.len(), 56);
        }
        {
            // Test with distinct.
            let select = Select::full::<Plan>();
            let select_ai = Select::full::<EcAgendaItem>()
                .distinct_on(&["source_uuid"])
                .add_replace_order("source_uuid", FieldSortKind::Asc)
                .add_replace_order("changed_at", FieldSortKind::Desc);
            let select_pi = Select::full::<EcProtocolItem>()
                .distinct_on(&["source_uuid"])
                .add_replace_order("source_uuid", FieldSortKind::Asc)
                .add_replace_order("changed_at", FieldSortKind::Desc);

            let h: Vec<JoinedPlan> = JoinedPlanSel::new(select)
                // .distinct()
                .set_agenda_item(EcAgendaItem::join_default().selecting(select_ai))
                .set_protocol_item(
                    EcProtocolItem::join_default().selecting(select_pi),
                )
                .add_order_asc("id")
                .get(&*pool)
                .await
                .unwrap();

            let h2 = h
                .iter()
                .map(|x| {
                    (
                        x.plan.id,
                        x.plan.uuid.to_string(),
                        x.agenda_item.number,
                        x.protocol_item.number,
                    )
                })
                .collect::<Vec<_>>();
            let theoretical_h2 = [
                (
                    1000019029,
                    "97f9539c-5683-11ed-8dc7-566ff2f30017".to_string(),
                    9000029243,
                    8000129243,
                ),
                (
                    1000019030,
                    "97f9539c-5683-11ed-8dc7-566ff2f30019".to_string(),
                    9000129243,
                    8000129243,
                ),
                (
                    1000019195,
                    "a9d54313-5a82-11ed-8198-566ff2f30018".to_string(),
                    9000229243,
                    8000129243,
                ),
                (
                    1000020496,
                    "df31f9f2-b890-4647-80f2-51eae1f2753d".to_string(),
                    9001529243,
                    8001529243,
                ),
                (
                    1000024820,
                    "566ff2f3-007a-1ede-85e6-3bf75cccde3b".to_string(),
                    9000929243,
                    8000929243,
                ),
                (
                    1000026781,
                    "2e788178-5206-11ee-8237-566ff2f30017".to_string(),
                    9000029243,
                    8000129243,
                ),
                (
                    1000029243,
                    "d6229360-06fc-11ee-805c-566ff2f30017".to_string(),
                    9001629243,
                    8001629243,
                ),
            ];
            assert_eq!(h.len(), 7);
            assert_eq!(&h2, &theoretical_h2, "{:#?}\n{:#?}", h2, theoretical_h2);
        }
    })
    .await
}
