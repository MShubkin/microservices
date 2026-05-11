use super::*;
use crate::app_process;
use crate::app_process::tests::run_db_test;

const GET_PLANS_EXTRA_MIGS: &[&str] = &["price_analysis/get_plans.sql"];

#[tokio::test]
async fn test_get_plans_attachments() {
    run_db_test(GET_PLANS_EXTRA_MIGS, |pool| async move {
        let input = CompletePlansRequest {
            section: Section::None,
            user_id: USER1,
            select: Select::full::<Plan>().eq(Plan::id, 1),
            item_fields: PlanItem::FIELDS.iter().map(|x| x.to_string()).collect(),
        };

        let r = app_process::get_complete_plans(input, pool).await.unwrap();

        // This check checks fields.
        assert!(!r.data.item_list.is_empty());
        let a = &r.data.item_list[0].attachments;
        println!("{a:#?}");
        assert_eq!(a.len(), 4);
        assert_eq!(a[0].id, Some(5));
        assert_eq!(a[0].category_id.map(|x| x as i16), Some(1));
        assert_eq!(a[1].id, Some(5));
        assert_eq!(a[1].category_id.map(|x| x as i16), Some(1));
        assert_eq!(a[2].id, Some(3));
        assert_eq!(a[2].category_id.map(|x| x as i16), Some(2));
        assert_eq!(a[3].id, Some(2));
        assert_eq!(a[3].category_id.map(|x| x as i16), Some(2));
    })
    .await
}
