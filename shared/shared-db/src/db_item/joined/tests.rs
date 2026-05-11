use super::*;
use crate::db_item::selection::*;

use crate as asez2_shared_db;
use crate::Value;

#[derive(Debug, DbItem, Clone, PartialEq, Default)]
#[item_table = "plan"]
pub struct Plan {
    #[item_field_pkey]
    id: i64,
    content: String,
}

#[derive(Debug, DbItem, Clone, PartialEq, Default)]
#[item_table = "plan_item"]
pub struct PlanItem {
    #[item_field_pkey]
    id: i64,
    plan_id: i64,
    content: String,
}

#[derive(Debug, DbItem, Clone, PartialEq, Default)]
#[item_table = "plan_owner"]
pub struct PlanOwner {
    #[item_field_pkey]
    id: i64,
    plan_id: i64,
    name: String,
    address_id: i64,
}

#[derive(Debug, DbItem, Clone, PartialEq, Default)]
#[item_table = "addresses"]
pub struct Address {
    #[item_field_pkey]
    id: i64,
    address: String,
}

#[derive(Debug, DbItem, Clone, PartialEq, Default)]
#[item_table = "plan_secret"]
pub struct PlanSecret {
    #[item_field_pkey]
    id: i64,
    plan_id: i64,
    code: String,
    description: String,
}

crate::impl_join_on!(PlanOwner:address_id => Address:id);
crate::impl_join_on!(Plan:id => PlanOwner:plan_id);
crate::impl_join_on!(Plan:id => PlanItem:plan_id, aggr);
crate::impl_join_on!(Plan:id => PlanSecret:plan_id, left);
crate::impl_join_on!(Plan:id => PlanSecret:plan_id, aggr);

crate::impl_join_on!(PlanItem:plan_id => PlanOwner:plan_id);
crate::impl_join_on!(PlanItem:plan_id => PlanOwner:plan_id, aggr);

crate::joined!(
    !PlanOwnerSLO,
    plan: Plan,
    plan_owner: PlanOwner[Plan => PlanOwner],
    plan_secret: PlanSecret[Plan => PlanSecret, left],
    item: PlanItem[Plan => PlanItem, aggr],
    // NB: This join does not join directly to plans! Here
    // we follow plan->plan_owner->address.
    address: Address[PlanOwner => Address],
);
crate::joined!(
    item: PlanItem,
    owner: PlanOwner[PlanItem => PlanOwner],
);
crate::joined!(
    !PlanWithItems,
    plan: Plan,
    items: PlanItem[Plan => PlanItem, aggr],
);
crate::joined!(
    !PlanWithItemsAndSecrets,
    plan: Plan,
    items: PlanItem[Plan => PlanItem, aggr],
    secrets: PlanSecret[Plan => PlanSecret, aggr],
);
mod aggr {
    use super::*;
    crate::joined!(
        !ItemWithOwners,
        item: PlanItem,
        owners: PlanOwner[PlanItem => PlanOwner, aggr],
    );
}

async fn fill_db(
    pool: &mut sqlx::Transaction<'_, Postgres>,
    file: &str,
) -> Result<()> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mig_path = std::path::PathBuf::from(&manifest)
        .join("src")
        .join("db_item")
        .join("joined")
        .join("test_sql")
        .join(file);
    let commands = std::fs::read_to_string(&mig_path)?;

    for command in commands.split_inclusive(';') {
        if let Err(e) = sqlx::query(command).execute(&mut *pool).await {
            println!("{:?}: {}", mig_path, e);
            break;
        }
    }
    Ok(())
}

impl PlanItem {
    fn new(id: i64, plan_id: i64, content: &str) -> Self {
        Self {
            id,
            plan_id,
            content: content.to_owned(),
        }
    }
}

impl PlanOwner {
    fn new(id: i64, plan_id: i64, name: &str, address_id: i64) -> Self {
        Self {
            id,
            plan_id,
            name: name.to_owned(),
            address_id,
        }
    }
}

#[tokio::test]
/// Aim to get the single entry for all OwnedPlans, with distinct
/// owned items and each one with the owner with the highest address.
async fn test_owner_address_order_distinct() {
    use self::JoinedPlanItemPlanOwner as OwnedItem;
    use self::JoinedPlanItemPlanOwnerSelector as OwnedItemSel;
    use crate::test_setup::run_db_test;

    run_db_test(
        "plan",
        "(id BIGINT NOT NULL, content TEXT NOT NULL)",
        Some("(id, content) values(5, 'stuff'),(7,'more stuff')"),
        |mut pool| async move {
            fill_db(&mut pool, "test_order.sql").await.unwrap();

            let ordered_item = Select::full::<PlanItem>();

            let ordered_owner = Select::full::<PlanOwner>()
                .add_replace_order("plan_id", FieldSortKind::Asc)
                .add_replace_order("address_id", FieldSortKind::Desc)
                .distinct_on(&["plan_id"]);

            let h = OwnedItemSel::new(ordered_item)
                .set_owner(PlanOwner::join_default().selecting(ordered_owner))
                .get(&mut pool)
                .await
                .unwrap();

            let expected = vec![
                OwnedItem {
                    item: PlanItem::new(2, 4, "awdadw"),
                    owner: PlanOwner::new(342, 4, "Колобок", 3),
                },
                OwnedItem {
                    item: PlanItem::new(3, 5, "another entry"),
                    owner: PlanOwner::new(344, 5, "Дед Мороз", 5),
                },
                OwnedItem {
                    item: PlanItem::new(3, 7, "ho ho ho"),
                    owner: PlanOwner::new(344, 7, "Дед Мороз", 7),
                },
            ];

            assert_eq!(h, expected);
        },
    )
    .await
}

#[tokio::test]
/// Aim to get the single entry for all OwnedPlans, with each
/// owned item. We also order them for the sake of sanity.
async fn test_owner_address_order() {
    use self::JoinedPlanItemPlanOwner as OwnedItem;
    use self::JoinedPlanItemPlanOwnerSelector as OwnedItemSel;
    use crate::test_setup::run_db_test;

    run_db_test(
        "plan",
        "(id BIGINT NOT NULL, content TEXT NOT NULL)",
        Some("(id, content) values(5, 'stuff'),(7,'more stuff')"),
        |mut pool| async move {
            fill_db(&mut pool, "test_order.sql").await.unwrap();

            let ordered_item = Select::full::<PlanItem>();
            let ordered_owner = Select::full::<PlanOwner>();

            let plan_owner = PlanOwner::join_default()
                .selecting(ordered_owner)
                .with_outer_order_desc("address_id")
                .unwrap()
                .with_outer_order_desc("name")
                .unwrap();

            let h = OwnedItemSel::new(ordered_item)
                .set_owner(plan_owner)
                .get(&mut pool)
                .await
                .unwrap();

            let expected = vec![
                OwnedItem {
                    item: PlanItem::new(3, 7, "ho ho ho"),
                    owner: PlanOwner::new(344, 7, "Дед Мороз", 7),
                },
                OwnedItem {
                    item: PlanItem::new(3, 7, "ho ho ho"),
                    owner: PlanOwner::new(345, 7, "Дед Мороз", 6),
                },
                OwnedItem {
                    item: PlanItem::new(3, 5, "another entry"),
                    owner: PlanOwner::new(344, 5, "Дед Мороз", 5),
                },
                OwnedItem {
                    item: PlanItem::new(3, 5, "another entry"),
                    owner: PlanOwner::new(343, 5, "Маленький Принц", 4),
                },
                OwnedItem {
                    item: PlanItem::new(3, 7, "ho ho ho"),
                    owner: PlanOwner::new(343, 7, "Маленький Принц", 3),
                },
                OwnedItem {
                    item: PlanItem::new(2, 4, "awdadw"),
                    owner: PlanOwner::new(342, 4, "Колобок", 3),
                },
                OwnedItem {
                    item: PlanItem::new(3, 5, "another entry"),
                    owner: PlanOwner::new(343, 5, "Маленький Принц", 2),
                },
                OwnedItem {
                    item: PlanItem::new(2, 4, "awdadw"),
                    owner: PlanOwner::new(342, 4, "Колобок", 2),
                },
                OwnedItem {
                    item: PlanItem::new(2, 4, "awdadw"),
                    owner: PlanOwner::new(342, 4, "Колобок", 1),
                },
            ];

            assert_eq!(h, expected);
        },
    )
    .await
}

#[tokio::test]
/// Aim to get the single entry for all OwnedPlans, with each
/// owned item. We also order them for the sake of sanity.
async fn test_owner_address_and_plan_id_order() {
    use self::JoinedPlanItemPlanOwner as OwnedItem;
    use self::JoinedPlanItemPlanOwnerSelector as OwnedItemSel;
    use crate::test_setup::run_db_test;

    run_db_test(
        "plan",
        "(id BIGINT NOT NULL, content TEXT NOT NULL)",
        Some("(id, content) values(5, 'stuff'),(7,'more stuff')"),
        |mut pool| async move {
            fill_db(&mut pool, "test_order.sql").await.unwrap();

            let ordered_item = Select::full::<PlanItem>();
            let ordered_owner = Select::full::<PlanOwner>();

            let plan_owner = PlanOwner::join_default()
                .selecting(ordered_owner)
                .with_outer_order_desc("address_id")
                .unwrap();

            let h = OwnedItemSel::new(ordered_item)
                .add_order_desc("plan_id")
                .set_owner(plan_owner)
                .get(&mut pool)
                .await
                .unwrap();

            let expected = vec![
                OwnedItem {
                    item: PlanItem::new(3, 7, "ho ho ho"),
                    owner: PlanOwner::new(344, 7, "Дед Мороз", 7),
                },
                OwnedItem {
                    item: PlanItem::new(3, 7, "ho ho ho"),
                    owner: PlanOwner::new(345, 7, "Дед Мороз", 6),
                },
                OwnedItem {
                    item: PlanItem::new(3, 7, "ho ho ho"),
                    owner: PlanOwner::new(343, 7, "Маленький Принц", 3),
                },
                OwnedItem {
                    item: PlanItem::new(3, 5, "another entry"),
                    owner: PlanOwner::new(344, 5, "Дед Мороз", 5),
                },
                OwnedItem {
                    item: PlanItem::new(3, 5, "another entry"),
                    owner: PlanOwner::new(343, 5, "Маленький Принц", 4),
                },
                OwnedItem {
                    item: PlanItem::new(3, 5, "another entry"),
                    owner: PlanOwner::new(343, 5, "Маленький Принц", 2),
                },
                OwnedItem {
                    item: PlanItem::new(2, 4, "awdadw"),
                    owner: PlanOwner::new(342, 4, "Колобок", 3),
                },
                OwnedItem {
                    item: PlanItem::new(2, 4, "awdadw"),
                    owner: PlanOwner::new(342, 4, "Колобок", 2),
                },
                OwnedItem {
                    item: PlanItem::new(2, 4, "awdadw"),
                    owner: PlanOwner::new(342, 4, "Колобок", 1),
                },
            ];

            assert_eq!(h, expected);
        },
    )
    .await
}

#[tokio::test]
/// Тест на проверку того, что aggr джойн селекты правильно сортируют данные
async fn test_owner_address_aggr_distinct_order() {
    use self::aggr::ItemWithOwners;
    use self::aggr::ItemWithOwnersSelector;
    use crate::test_setup::run_db_test;

    run_db_test(
        "plan",
        "(id BIGINT NOT NULL, content TEXT NOT NULL)",
        None,
        |mut pool| async move {
            fill_db(&mut pool, "test_order.sql").await.unwrap();

            let ordered_item = Select::full::<PlanItem>()
                // This is needed for distinct to work.
                .add_replace_order_asc("id")
                // we need both this clause for plan_id and
                // the `add_order_desc("plan_id")` on the outermost layer
                .add_replace_order_desc("plan_id")
                .distinct_on(&["id"]);
            let ordered_owner =
                Select::full::<PlanOwner>().add_replace_order_desc("id");

            let h = ItemWithOwnersSelector::new(ordered_item)
                .set_owners(PlanOwner::join_default().selecting(ordered_owner))
                // we need both this clause for plan_id and
                // the `add_replace_order_desc("plan_id")` on the innermost layer.
                .add_order_desc("plan_id")
                .get(&mut pool)
                .await
                .unwrap();

            let expected = vec![
                ItemWithOwners {
                    item: PlanItem {
                        id: 3,
                        plan_id: 7,
                        content: String::from("ho ho ho"),
                    },
                    owners: vec![
                        PlanOwner {
                            id: 345,
                            plan_id: 7,
                            name: String::from("Дед Мороз"),
                            address_id: 6,
                        },
                        PlanOwner {
                            id: 344,
                            plan_id: 7,
                            name: String::from("Дед Мороз"),
                            address_id: 7,
                        },
                        PlanOwner {
                            id: 343,
                            plan_id: 7,
                            name: String::from("Маленький Принц"),
                            address_id: 3,
                        },
                    ],
                },
                ItemWithOwners {
                    item: PlanItem {
                        id: 2,
                        plan_id: 4,
                        content: String::from("awdadw"),
                    },
                    owners: vec![
                        PlanOwner {
                            id: 342,
                            plan_id: 4,
                            name: String::from("Колобок"),
                            address_id: 1,
                        },
                        PlanOwner {
                            id: 342,
                            plan_id: 4,
                            name: String::from("Колобок"),
                            address_id: 2,
                        },
                        PlanOwner {
                            id: 342,
                            plan_id: 4,
                            name: String::from("Колобок"),
                            address_id: 3,
                        },
                    ],
                },
                ItemWithOwners {
                    item: PlanItem {
                        id: 1,
                        plan_id: 3,
                        content: String::from("stuffing"),
                    },
                    owners: vec![],
                },
            ];

            assert_eq!(h, expected);
        },
    )
    .await
}

#[tokio::test]
async fn test_joined_plan() {
    use self::PlanOwnerSLO as JoinedPlan;
    use self::PlanOwnerSLOSelector as JoinedPlanSelector;
    use crate::test_setup::run_db_test;

    run_db_test(
        "plan",
        "(id BIGINT NOT NULL, content TEXT NOT NULL)",
        Some("(id, content) values(5, 'stuff'),(7,'more stuff')"),
        |mut pool| async move {
            fill_db(&mut pool, "test.sql").await.unwrap();

            let select = Select::full_in::<_, Plan>("id", vec![Value::Int(5)]);
            let legacy = Select::full::<PlanItem>();

            let legacy_item_selector = PlanItem::join_default()
                .selecting(legacy)
                .add_order_aggr_asc_by("content")
                .unwrap();

            let h = JoinedPlanSelector::new(select)
                .set_item(legacy_item_selector)
                .distinct()
                .get(&mut pool)
                .await
                .unwrap();

            let plan = Plan {
                id: 5,
                content: String::from("stuff"),
            };
            let plan_secret = Some(PlanSecret {
                id: 999,
                plan_id: 5,
                code: "006".to_string(),
                description: "I dislike James Bond".to_string(),
            });
            let plan_owner = PlanOwner {
                id: 342,
                plan_id: 5,
                name: "Колобок".to_string(),
                address_id: 1,
            };
            let address = Address {
                id: 1,
                address: "У бабушки и дедушки".to_string(),
            };
            let i = PlanItem {
                id: 1,
                plan_id: 5,
                content: String::from("stuffing"),
            };
            let i2 = PlanItem {
                id: 2,
                plan_id: 5,
                content: String::from("awdadw"),
            };
            let i3 = PlanItem {
                id: 3,
                plan_id: 5,
                content: String::from("another entry"),
            };
            let j = JoinedPlan {
                plan,
                plan_secret,
                plan_owner,
                address,
                item: vec![i3, i2, i],
            };
            assert_eq!(vec![j], h);
        },
    )
    .await
}

#[tokio::test]
async fn test_joined_plan_binds_in_subselects() {
    use self::PlanOwnerSLO as JoinedPlan;
    use self::PlanOwnerSLOSelector as JoinedPlanSelector;
    use crate::test_setup::run_db_test;

    run_db_test(
        "plan",
        "(id BIGINT NOT NULL, content TEXT NOT NULL)",
        Some("(id, content) values(5, 'stuff'),(7,'more stuff')"),
        |mut pool| async move {
            fill_db(&mut pool, "test.sql").await.unwrap();

            let select = Select::full_in::<_, Plan>("id", vec![Value::Int(5)])
                .add_replace_order("id", FieldSortKind::Asc);

            // This select demonstrates that all fields will be taken anyway.
            let select_abuse = Select::with_fields(["id"])
                .add_expand_filter("id", SelectionKind::In, vec![Value::Int(5)])
                .add_replace_order("id", FieldSortKind::Asc);

            let item_select = Select::full_in::<_, PlanItem>(
                "id",
                vec![Value::Int(2), Value::Int(1)],
            )
            .add_replace_order("id", FieldSortKind::Asc);

            let secret_select =
                Select::full_in::<_, PlanSecret>("code", vec![Value::from("red")]);

            let owner_select = Select::full::<PlanOwner>();

            let item_select = PlanItem::join_on("id")
                .eq_own("plan_id")
                .add_order_aggr_asc_by("content")
                .unwrap()
                .selecting(item_select);
            let owner_select =
                PlanOwner::join_on("id").eq_own("plan_id").selecting(owner_select);
            let secret_select = PlanSecret::join_on("id")
                .eq_own("plan_id")
                .selecting(secret_select);

            let h = JoinedPlanSelector::new(select)
                .set_item(item_select.clone())
                .set_plan_owner(owner_select.clone())
                .set_plan_secret(secret_select.clone())
                .distinct()
                .get(&mut pool)
                .await
                .unwrap();

            let h_abuse = JoinedPlanSelector::new(select_abuse)
                .set_item(item_select)
                .set_plan_owner(owner_select)
                .set_plan_secret(secret_select)
                .distinct()
                .get(&mut pool)
                .await
                .unwrap();

            let plan = Plan {
                id: 5,
                content: String::from("stuff"),
            };
            let plan_owner = PlanOwner {
                id: 342,
                plan_id: 5,
                name: "Колобок".to_string(),
                address_id: 1,
            };
            let address = Address {
                id: 1,
                address: "У бабушки и дедушки".to_string(),
            };
            let i = PlanItem {
                id: 1,
                plan_id: 5,
                content: String::from("stuffing"),
            };
            let i2 = PlanItem {
                id: 2,
                plan_id: 5,
                content: String::from("awdadw"),
            };
            let j = JoinedPlan {
                plan,
                plan_secret: None,
                plan_owner,
                address,
                item: vec![i2, i],
            };
            assert_eq!(vec![j], h);
            assert_eq!(h_abuse, h, "Our system is abusable!");
        },
    )
    .await
}
#[tokio::test]
async fn test_joined_aggr_duplicates() {
    use self::JoinedPlanPlanItemPlanSecretSelector as PlanItemsSecretsSelector;
    use crate::test_setup::run_db_test;

    run_db_test(
        "plan",
        "(id BIGINT NOT NULL, content TEXT NOT NULL)",
        Some("(id, content) values(1, 'stuff')"),
        |mut pool| async move {
            fill_db(&mut pool, "test_aggr.sql").await.unwrap();

            let joined_plan_dups = {
                let select = Select::full_in::<_, Plan>("id", vec![Value::Int(1)])
                    .add_replace_order("id", FieldSortKind::Asc);

                let items = PlanItem::join_default();
                let secrets = PlanSecret::join_default();
                PlanItemsSecretsSelector::new(select)
                    .set_items(items)
                    .set_secrets(secrets)
                    .get(&mut pool)
                    .await
                    .unwrap()
                    .pop()
                    .unwrap()
            };

            let mut items = joined_plan_dups.items;
            let mut secrets = joined_plan_dups.secrets;

            let items_len = items.len();
            items.sort_by_key(|x| x.id);
            items.dedup();
            assert_ne!(items.len(), items_len);

            let secrets_len = secrets.len();
            secrets.sort_by_key(|x| x.id);
            secrets.dedup();
            assert_ne!(secrets.len(), secrets_len);

            let joined_plan = {
                let select = Select::full_in::<_, Plan>("id", vec![Value::Int(1)])
                    .add_replace_order("id", FieldSortKind::Asc);

                let items = PlanItem::join_default()
                    .selecting(Select::default().add_replace_order_asc("id"))
                    .distinct_aggr(true);
                let secrets = PlanSecret::join_default()
                    .selecting(Select::default().add_replace_order_asc("id"))
                    .distinct_aggr(true);
                PlanItemsSecretsSelector::new(select)
                    .set_items(items)
                    .set_secrets(secrets)
                    .get(&mut pool)
                    .await
                    .unwrap()
                    .pop()
                    .unwrap()
            };

            let mut items = joined_plan.items;
            let mut secrets = joined_plan.secrets;

            let items_len = items.len();
            items.sort_by_key(|x| x.id);
            items.dedup();
            assert_eq!(items_len, 6);
            assert_eq!(items.len(), items_len);

            let secrets_len = secrets.len();
            secrets.sort_by_key(|x| x.id);
            secrets.dedup();
            assert_eq!(secrets_len, 6);
            assert_eq!(secrets.len(), secrets_len);
        },
    )
    .await
}

mod extra {
    use crate::db_item::selection::*;
    use crate::db_item::*;
    use crate::joined;
    use crate::test_setup::run_db_test;

    use crate as asez2_shared_db;

    #[derive(Debug, DbItem, Clone, PartialEq, Default)]
    #[item_table = "plan"]
    pub struct Plan {
        #[item_field_pkey]
        id: i64,
        content: String,
    }

    #[derive(Debug, DbItem, Clone, PartialEq, Default)]
    #[item_table = "plan_item"]
    pub struct PlanItem {
        #[item_field_pkey]
        id: i64,
        plan_id: i64,
        content: String,
    }

    #[derive(Debug, DbItem, Clone, PartialEq, Default)]
    #[item_table = "plan_owner"]
    pub struct PlanOwner {
        #[item_field_pkey]
        id: i64,
        plan_id: i64,
        name: String,
    }

    #[derive(Debug, DbItem, Clone, PartialEq, Default)]
    #[item_table = "addresses"]
    pub struct Address {
        #[item_field_pkey]
        id: i64,
        object_id: i64,
        address: String,
    }

    #[derive(Debug, DbItem, Clone, PartialEq, Default)]
    #[item_table = "secrets"]
    pub struct Secret {
        #[item_field_pkey]
        id: i64,
        object_id: i64,
        code: String,
    }

    crate::impl_join_on!(Plan:id => PlanOwner:plan_id, aggr);
    crate::impl_join_on!(Plan:id => PlanItem:plan_id, aggr);
    crate::impl_join_on!(PlanOwner:id => Address:object_id, aggr);
    crate::impl_join_on!(PlanOwner:id => Secret:object_id, aggr);

    joined!(
        !JoinedComplex,
        header: Plan,
        items: PlanItem[Plan => PlanItem, aggr],
        owners: PlanOwner[Plan => PlanOwner, aggr],
        address: Address[PlanOwner => Address, aggr],
        secrets: Secret[PlanOwner => Secret, aggr],
    );

    #[tokio::test]
    async fn test_indirect_join() {
        use crate::test_setup::run_db_test;

        run_db_test(
            "plan",
            "(id BIGINT NOT NULL, content TEXT NOT NULL)",
            Some("(id, content) values(1, 'stuff'),(2, 'more stuff')"),
            |mut pool| async move {
                crate::db_item::joined::tests::fill_db(&mut pool, "test_aggr2.sql")
                    .await
                    .unwrap();

                let plan_select = Select::default().add_replace_order_asc("id");
                let plans = JoinedComplexSelector::new(plan_select)
                    .add_order_asc("id")
                    .get(&mut pool)
                    .await
                    .unwrap();

                assert_eq!(plans.len(), 2);
                assert_eq!(plans[0].header.id, 1);
                assert_eq!(plans[0].items.len(), 44);
                assert_eq!(plans[0].owners.len(), 44);
                assert_eq!(plans[0].address.len(), 44);
                assert_eq!(plans[0].secrets.len(), 44);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_indirect_join_dedup() {
        use crate::db_item::joined::JoinTo;
        use crate::test_setup::run_db_test;

        run_db_test(
            "plan",
            "(id BIGINT NOT NULL, content TEXT NOT NULL)",
            Some("(id, content) values(1, 'stuff'),(2, 'more stuff')"),
            |mut pool| async move {
                crate::db_item::joined::tests::fill_db(&mut pool, "test_aggr2.sql")
                    .await
                    .unwrap();

                let plan_select = Select::default().add_replace_order_asc("id");
                let plans = JoinedComplexSelector::new(plan_select)
                    .set_items(PlanItem::join_default().distinct_aggr(true))
                    .set_owners(PlanOwner::join_default().distinct_aggr(true))
                    .set_address(Address::join_default().distinct_aggr(true))
                    .set_secrets(
                        Secret::join_default()
                            .selecting(
                                Select::default().add_replace_order_desc("id"),
                            )
                            .distinct_aggr(true),
                    )
                    .add_order_asc("id")
                    // .distinct()
                    .get(&mut pool)
                    .await
                    .unwrap();

                assert_eq!(plans.len(), 2);
                assert_eq!(plans[0].header.id, 1);
                assert_eq!(plans[0].items.len(), 2);
                assert_eq!(plans[0].owners.len(), 2);
                assert_eq!(plans[0].address.len(), 4);
                assert_eq!(plans[0].secrets.len(), 11);

                let exp_ids = (7..18i64).rev().collect::<Vec<_>>();
                let got_ids =
                    plans[0].secrets.iter().map(|x| x.id).collect::<Vec<_>>();
                assert_ne!(exp_ids, got_ids);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_new_with_order() {
        run_db_test(
            "plan",
            "(id BIGINT NOT NULL, content TEXT NOT NULL)",
            Some("(id, content) values(1, 'stuff'),(2, 'more stuff')"),
            |mut pool| async move {
                crate::db_item::joined::tests::fill_db(&mut pool, "test_aggr2.sql")
                    .await
                    .unwrap();

                let plan_select = Select::default().add_replace_order_desc("id");
                let plans = JoinedComplexSelector::new_with_order(plan_select)
                    .get(&mut pool)
                    .await
                    .unwrap();

                assert_eq!(plans.len(), 2);
                assert_eq!(plans[0].header.id, 2);
                assert_eq!(plans[1].header.id, 1);
            },
        )
        .await
    }
}

mod pagination {
    use super::*;
    use crate::{db_item::Select, test_setup::run_db_test};

    #[tokio::test]
    async fn paginated_aggr() {
        run_db_test(
            "plan",
            "(id BIGINT NOT NULL, content TEXT NOT NULL)",
            Some("(id, content) values(1, 'stuff'),(2, 'more stuff'),(3, 'even more stuff')"),
            |mut pool| async move {
                crate::db_item::joined::tests::fill_db(&mut pool, "test_aggr2.sql")
                    .await
                    .unwrap();

                let plan_select = Select::default().offset(0).take_n(2).add_replace_order_desc(Plan::id);
                let items_select = Select::default().offset(0).take_n(2).add_replace_order_asc(PlanItem::id);
                let plans = PlanWithItemsSelector::new(plan_select)
                    .set_items(PlanItem::join_default().selecting(items_select))
                    .add_order_desc(Plan::id)
                    .get(&mut pool)
                    .await
                    .unwrap();

                assert_eq!(plans.len(), 2);
                assert_eq!(plans[0].plan.id, 3);
                assert_eq!(plans[0].items.len(), 1);
                assert_eq!(plans[1].plan.id, 2);
                assert_eq!(plans[1].items.len(), 3);
            },
        )
        .await
    }
}
