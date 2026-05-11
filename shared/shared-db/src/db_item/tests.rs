#![allow(clippy::field_reassign_with_default)]

use super::*;
use crate::test_setup::run_db_test;
use crate::uuid;

use serde::{Deserialize, Serialize};
use shared_db_derive::{DbAdaptor, DbItem};
use sqlx::FromRow;
use time::macros::{date, datetime};
use uuid::Uuid;

use crate as asez2_shared_db;

macro_rules! derive_tests {
    ($($aggr_arrays:ident)?) => {
mod timelord {
    use super::*;
    use crate::db_item::selection::*;
    use crate::Value;

    #[derive(Default, Debug, Clone, DbItem, PartialEq)]
    #[item_table = "timelord"]
    $(#[$aggr_arrays])?
    pub struct Timelord {
        #[item_field_pkey]
        id: AsezTimestamp,
        name: AsezDate,
        weight: Option<AsezDate>,
        time: Option<AsezTimestamp>,
    }

    #[tokio::test]
    async fn test_insert_date_time() {
        run_db_test(
            "timelord",
            "(id timestamp NOT NULL, name date NOT NULL, weight date, time timestamp)",
            None,
            |mut pool| async move {
                let mut timelord = Timelord {
                    id: AsezTimestamp::from(datetime!(1901-01-02 01:02:57)),
                    name: AsezDate::from(date!(0001-01-01)),
                    weight: Some(AsezDate::from(date!(1066-01-01))),
                    time: Some(AsezTimestamp::from(datetime!(2000-01-01 00:00:00))),
                };
                let ret = timelord.insert_returning(&mut pool).await.unwrap();

                assert_eq!(timelord, ret);
        }).await
    }

    #[tokio::test]
    async fn filter_date() {
        run_db_test(
            "timelord",
            "(id timestamp NOT NULL, name date NOT NULL, weight date, time timestamp)",
            Some(
                "(id, name) VALUES\
                ('1901-01-02 23:59:56','0001-01-01'),\
                ('1901-01-02 23:59:57','0100-01-01'),\
                ('1901-01-02 23:59:58','1501-05-01'),\
                ('1901-01-02 23:59:59','1501-05-12')"
            ),
            |mut pool| async move {
                let s = Select::with_fields::<&str, _>([])
                    .add_expand_filter("name", SelectionKind::Between, vec![
                        Value::from("01.01.0001"),
                        Value::from("02.05.1501"),
                    ])
                    .add_replace_order("name", FieldSortKind::Asc);
                let s2 = Select::with_fields::<&str, _>([])
                    .add_expand_filter("name", SelectionKind::Between, vec![
                        Value::from("01.01.0001"),
                        Value::from("02.05.1501"),
                    ])
                    .add_replace_order("name", FieldSortKind::Asc);

                let lords = Timelord::select(&s, &mut pool).await.unwrap();
                assert_eq!(lords.len(), 3);
                assert_eq!(&lords[0].name.to_string(), "01.01.0001");
                assert_eq!(&lords[1].name.to_string(), "01.01.0100");
                assert_eq!(&lords[2].name.to_string(), "01.05.1501");

                let lords = Timelord::select(&s2, &mut pool).await.unwrap();
                assert_eq!(lords.len(), 3);
                assert_eq!(&lords[0].name.to_string(), "01.01.0001");
                assert_eq!(&lords[1].name.to_string(), "01.01.0100");
                assert_eq!(&lords[2].name.to_string(), "01.05.1501");
        }).await
    }
}

mod animal_tests {
    use super::*;

    #[derive(Default, Debug, Clone, DbItem, DbAdaptor, PartialEq)]
    #[adaptor_derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
    #[item_table = "animals"]
    $(#[$aggr_arrays])?
    pub struct Animal {
        #[item_field_pkey]
        id: Uuid,
        #[adaptor_attributes(#[serde(default)])]
        name: String,
        weight_g: i32,
        habitat: Option<String>,
    }

    #[tokio::test]
    async fn test_insert_animal() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            None,
            |mut pool| async move {
                let hedgehog = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: Some("Hedgehog".to_string()),
                    weight_g: Some(1_500),
                    habitat: Some(None),
                };
                let part_hedgehog = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: None,
                    weight_g: Some(1_500),
                    habitat: None,
                };

                let c = hedgehog
                    .clone()
                    .into_item()
                    .unwrap()
                    .insert(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(c, 1);

                let animals = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                let exp_animal = Animal {
                    id: Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap(),
                    name: "Hedgehog".to_string(),
                    weight_g: 1_500,
                    habitat: None,
                };

                let rep = AnimalRep::from_item::<&str>(exp_animal.clone(), None);
                let rep2 = AnimalRep::from_item(
                    exp_animal,
                    Some(&["id", "weight_g"])
                );

                assert_eq!(animals.len(), 1);
                assert_eq!(rep, hedgehog, "full hedgehog failed.");
                assert_eq!(rep2, part_hedgehog);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_insert_animal_returning() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            None,
            |mut pool| async move {
                let hedgehog = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: Some("Hedgehog".to_string()),
                    weight_g: Some(1_500),
                    habitat: Some(None),
                };

                let output = hedgehog
                    .clone()
                    .into_item()
                    .unwrap()
                    .insert_returning(&mut pool)
                    .await
                    .unwrap();

                let mut animals = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(animals.len(), 1);
                let inserted = animals.pop().unwrap();

                let exp_animal = Animal {
                    id: Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap(),
                    name: "Hedgehog".to_string(),
                    weight_g: 1_500,
                    habitat: None,
                };
                assert_eq!(output, exp_animal);
                assert_eq!(inserted, exp_animal);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_insert_vec() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            None,
            |mut pool| async move {
                let hedgehog = AnimalRep {
                    id: Some(Uuid::parse_str("522ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: Some("Hedgehog".to_string()),
                    weight_g: Some(1_500),
                    habitat: Some(None),
                };
                let fox = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-1178-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: Some("Fox".to_string()),
                    weight_g: Some(20_000),
                    habitat: Some(Some("In your chicken coop".to_string())),
                };
                let bear = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-0078-1ebb-89bf-a52f40e61a8d").unwrap()),
                    name: Some("Bear".to_string()),
                    weight_g: Some(300_000),
                    habitat: Some(Some("Where you least expect it".to_string())),
                };

                let animal_reps = vec![hedgehog, fox, bear];
                let mut animals: Vec<_> = animal_reps
                    .iter()
                    .cloned()
                    .map(|x| x.into_item().unwrap())
                    .collect();

                let c = Animal::insert_vec(&mut animals, &mut pool).await.unwrap();
                assert_eq!(c, 3);

                let got_animals = sqlx::query("SELECT * FROM animals ORDER BY weight_g ASC;")
                    .map(|x| AnimalRep::from_item::<&str>(
                        Animal::from_row(&x).unwrap(),
                        None
                    ))
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                assert_eq!(got_animals.len(), 3);
                assert_eq!(got_animals, animal_reps);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_insert_vec_returning() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            None,
            |mut pool| async move {
                let hedgehog = Animal {
                    id: Uuid::parse_str("522ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap(),
                    name: "Hedgehog".to_string(),
                    weight_g: 1_500,
                    habitat: None,
                };
                let fox = Animal {
                    id: Uuid::parse_str("566ff2f3-1178-1eee-89bf-a52f40e61a8d").unwrap(),
                    name: "Fox".to_string(),
                    weight_g: 20_000,
                    habitat: Some("In your chicken coop".to_string()),
                };
                let bear = Animal {
                    id: Uuid::parse_str("566ff2f3-0078-1ebb-89bf-a52f40e61a8d").unwrap(),
                    name: "Bear".to_string(),
                    weight_g: 300_000,
                    habitat: Some("Where you least expect it".to_string()),
                };

                let mut animals = vec![hedgehog, fox, bear];
                let output = Animal::insert_vec_returning(&mut animals, &mut pool)
                    .await
                    .unwrap();

                let got_animals = sqlx::query("SELECT * FROM animals ORDER BY weight_g ASC;")
                    .try_map(|x| Animal::from_row(&x))
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                assert_eq!(got_animals, animals);
                assert_eq!(output, animals);
            },
        )
        .await;
    }

    #[tokio::test]
    /// We insert so many animals that things have to get chunked and see if we get errors.
    async fn test_chunked_insert_vec() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            None,
            |mut pool| async move {
                let mut animal_reps = Vec::with_capacity(36_000);
                for i in 0..36_000 {
                    animal_reps.push(AnimalRep {
                        id: Some(Uuid::new_v4()),
                        name: Some(format!("Animal_{}", i)),
                        weight_g: Some(i),
                        habitat: Some(None),
                    });
                }
                let mut animals: Vec<_> = animal_reps
                    .iter()
                    .cloned()
                    .map(|x| x.into_item().unwrap())
                    .collect();

                let c = Animal::insert_vec(&mut animals, &mut pool).await.unwrap();
                assert_eq!(c, 36_000);

                let got_animals = sqlx::query("SELECT * FROM animals ORDER BY weight_g ASC;")
                    .map(|x| AnimalRep::from_item::<&str>(
                        Animal::from_row(&x).unwrap(),
                        None
                    ))
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                assert_eq!(got_animals.len(), 36_000);
                assert!(got_animals == animal_reps);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_update_animal() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g) values('566ff2f3-0078-1eee-89bf-a52f40e61a8d', 'mouse', 50)",
            ),
            |mut pool| async move {
                let update_mouse_exp = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: Some("mouse".to_string()),
                    weight_g: Some(50),
                    habitat: Some(Some("house".to_owned())),
                };
                let exp_old_mouse = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: Some("mouse".to_string()),
                    weight_g: Some(50),
                    habitat: Some(None),
                };

                let mut ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(ret.len(), 1);

                let ret = AnimalRep::from_item::<&str>(ret.pop().unwrap(), None);
                assert_eq!(ret, exp_old_mouse);

                let r = update_mouse_exp
                    .clone()
                    .into_item()
                    .unwrap()
                    .update(None, &mut pool)
                    .await
                    .unwrap();
                assert_eq!(r, 1);

                let mut ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(ret.len(), 1);

                let ret = AnimalRep::from_item::<&str>(ret.pop().unwrap(), None);
                assert_eq!(ret, update_mouse_exp);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_update_animal_returning() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g) values('566ff2f3-0078-1eee-89bf-a52f40e61a8d', 'mouse', 50)",
            ),
            |mut pool| async move {
                let update_mouse_exp = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: Some("mouse".to_string()),
                    weight_g: Some(50),
                    habitat: Some(Some("house".to_owned())),
                };
                let exp_old_mouse = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: Some("mouse".to_string()),
                    weight_g: Some(50),
                    habitat: Some(None),
                };

                let mut ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(ret.len(), 1);

                let ret = AnimalRep::from_item::<&str>(ret.pop().unwrap(), None);
                assert_eq!(ret, exp_old_mouse);

                let updatable = update_mouse_exp
                    .clone()
                    .into_item()
                    .unwrap();

                let r = updatable
                    .update_returning::<_, &str>(None, None, &mut pool)
                    .await
                    .unwrap();
                assert_eq!(r, updatable);

                let mut ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(ret.len(), 1);

                let ret = AnimalRep::from_item::<&str>(ret.pop().unwrap(), None);
                assert_eq!(ret, update_mouse_exp);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_update_single_animal_as_vec() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g) values('566ff2f3-0078-1eee-89bf-a52f40e61a8d', 'mouse', 50)",
            ),
            |mut pool| async move {
                let update_mouse_exp = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: Some("mouse".to_string()),
                    weight_g: Some(50),
                    habitat: Some(Some("house".to_owned())),
                };
                let exp_old_mouse = AnimalRep {
                    id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                    name: Some("mouse".to_string()),
                    weight_g: Some(50),
                    habitat: Some(None),
                };

                let mut ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(ret.len(), 1);

                let ret = AnimalRep::from_item::<&str>(ret.pop().unwrap(), None);
                assert_eq!(ret, exp_old_mouse);

                let update = vec![update_mouse_exp.clone().into_item().unwrap()];
                let r = DbItem::update_vec(&update, None, &mut pool).await.unwrap();
                assert_eq!(r, 1);

                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|x| AnimalRep::from_item::<&str>(x, None))
                    .collect::<Vec<_>>();

                assert_eq!(ret.len(), 1);
                assert_eq!(ret, vec![update_mouse_exp]);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_update_animals_as_vec_all_fields() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g) values('566ff2f3-0078-1eee-89bf-a52f40e61a8d', 'mouse', 50),
            ('566ff2f3-0077-1eee-89bf-a52f40e61a8d', 'rat', 500),
            ('566ff2f3-0076-1eee-89bf-a52f40e61a8d', 'capybara', 50000)",
            ),
            |mut pool| async move {
                let upd = vec![
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("mouse".to_string()),
                        weight_g: Some(50),
                        habitat: Some(Some("field".to_owned())),
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0077-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("rat".to_string()),
                        weight_g: Some(500),
                        habitat: Some(Some("house".to_owned())),
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0076-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("capybara".to_string()),
                        weight_g: Some(50000),
                        habitat: Some(None),
                    },
                ];

                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|x| AnimalRep::from_item::<&str>(x, None))
                    .collect::<Vec<_>>();
                assert_eq!(ret.len(), 3);

                assert_ne!(ret, upd);

                let upd_items = upd
                    .iter()
                    .cloned()
                    .map(|x| x.into_item().unwrap())
                    .collect::<Vec<_>>();
                let r = DbItem::update_vec(&upd_items, None, &mut pool).await.unwrap();
                assert_eq!(r, 3);

                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|x| AnimalRep::from_item::<&str>(x, None))
                    .collect::<Vec<_>>();

                assert_eq!(ret.len(), 3);
                assert_eq!(ret, upd);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_update_animals_as_vec_all_fields_returning() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g) values('566ff2f3-0078-1eee-89bf-a52f40e61a8d', 'mouse', 50),
            ('566ff2f3-0077-1eee-89bf-a52f40e61a8d', 'rat', 500),
            ('566ff2f3-0076-1eee-89bf-a52f40e61a8d', 'capybara', 50000)",
            ),
            |mut pool| async move {
                let upd = vec![
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("mouse".to_string()),
                        weight_g: Some(50),
                        habitat: Some(Some("field".to_owned())),
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0077-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("rat".to_string()),
                        weight_g: Some(500),
                        habitat: Some(Some("house".to_owned())),
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0076-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("capybara".to_string()),
                        weight_g: Some(50000),
                        habitat: Some(None),
                    },
                ];
                let upd_comp = upd
                    .iter()
                    .cloned()
                    .map(|x| x.into_item().unwrap())
                    .collect::<Vec<_>>();

                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(ret.len(), 3);

                assert_ne!(ret, upd_comp);

                let update_ret = DbItem::update_vec_returning::<&str>(
                    &upd_comp,
                    None,
                    None,
                    &mut pool
                ).await.unwrap();

                assert_eq!(update_ret.len(), 3);
                assert_eq!(update_ret, upd_comp);

                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                assert_eq!(ret.len(), 3);
                assert_eq!(ret, upd_comp);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_update_animals_as_vec_all_fields_returning_name() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g) values('566ff2f3-0078-1eee-89bf-a52f40e61a8d', 'mouse', 50),
            ('566ff2f3-0077-1eee-89bf-a52f40e61a8d', 'rat', 500),
            ('566ff2f3-0076-1eee-89bf-a52f40e61a8d', 'capybara', 50000)",
            ),
            |mut pool| async move {
                let upd = vec![
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("mouse".to_string()),
                        weight_g: Some(50),
                        habitat: Some(Some("field".to_owned())),
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0077-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("rat".to_string()),
                        weight_g: Some(500),
                        habitat: Some(Some("house".to_owned())),
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0076-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("capybara".to_string()),
                        weight_g: Some(50000),
                        habitat: Some(None),
                    },
                ];
                let upd_comp = upd
                    .iter()
                    .cloned()
                    .map(|x| x.into_item().unwrap())
                    .collect::<Vec<_>>();

                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(ret.len(), 3);

                assert_ne!(ret, upd_comp);

                let update_ret = DbItem::update_vec_returning::<&str>(
                    &upd_comp,
                    None,
                    Some(&["name"]),
                    &mut pool
                ).await.unwrap();

                assert_eq!(update_ret.len(), 3);
                for (upd, upd_origin) in update_ret.iter().zip(upd_comp.iter()) {
                    assert_eq!(upd.name, upd_origin.name);
                    assert_eq!(upd.id, upd_origin.id); // pkey is always returned.
                    assert_eq!(upd.weight_g, i32::default());
                    assert_eq!(upd.habitat, Default::default());
                }

                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                assert_eq!(ret.len(), 3);
                assert_eq!(ret, upd_comp);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_update_animals_as_vec_name_habitat_only() {
        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g) values('566ff2f3-0078-1eee-89bf-a52f40e61a8d', 'mouse', 50),
            ('566ff2f3-0077-1eee-89bf-a52f40e61a8d', 'rat', 500),
            ('566ff2f3-0076-1eee-89bf-a52f40e61a8d', 'capybara', 50000)",
            ),
            |mut pool| async move {
                let mut upd = vec![
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("mouse".to_string()),
                        weight_g: Some(50),
                        habitat: Some(Some("field".to_owned())),
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0077-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("rat".to_string()),
                        weight_g: Some(500),
                        habitat: Some(Some("house".to_owned())),
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0076-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("capybara".to_string()),
                        weight_g: Some(-50000),
                        habitat: Some(None),
                    },
                ];
                let exp_partial = vec![
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("mouse".to_string()),
                        weight_g: None,
                        habitat: None,
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0077-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("rat".to_string()),
                        weight_g: None,
                        habitat: None,
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0076-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("capybara".to_string()),
                        weight_g: None,
                        habitat: None,
                    },
                ];

                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|x| AnimalRep::from_item::<&str>(x, None))
                    .collect::<Vec<_>>();
                assert_eq!(ret.len(), 3);

                assert_ne!(ret, upd);

                let upd_items = upd
                    .iter()
                    .cloned()
                    .map(|x| x.into_item().unwrap())
                    .collect::<Vec<_>>();
                let r = DbItem::update_vec(
                    &upd_items,
                    Some(&["name", "habitat"]),
                    &mut pool
                ).await.unwrap();

                assert_eq!(r, 3);

                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|x| AnimalRep::from_item::<&str>(x, None))
                    .collect::<Vec<_>>();

                upd[2].weight_g = Some(50000);
                assert_eq!(ret.len(), 3);
                assert_eq!(ret, upd);

                let partial_ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|x| AnimalRep::from_item(x, Some(&["id", "name"])))
                    .collect::<Vec<_>>();
                assert_eq!(partial_ret, exp_partial);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_select_animals() {
        use crate::db_item::selection::*;
        use crate::Value;

        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g) values('566ff2f3-0078-1eee-89bf-a52f40e61a8d', 'mouse', 50),
            ('566ff2f3-0077-1eee-89bf-a52f40e61a8d', 'rat', 500),
            ('566ff2f3-0076-1eee-89bf-a52f40e61a8d', 'capybara', 50000)",
            ),
            |mut pool| async move {
                let exp = vec![
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("mouse".to_string()),
                        weight_g: Some(50),
                        habitat: Some(None),
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0077-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("rat".to_string()),
                        weight_g: Some(500),
                        habitat: Some(None),
                    },
                    AnimalRep {
                        id: Some(Uuid::parse_str("566ff2f3-0076-1eee-89bf-a52f40e61a8d").unwrap()),
                        name: Some("capybara".to_string()),
                        weight_g: Some(50000),
                        habitat:Some(None),
                    },
                ];

                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|x| AnimalRep::from_item::<&str>(x, None))
                    .collect::<Vec<_>>();

                assert_eq!(ret.len(), 3);
                assert_eq!(ret, exp);

                let selection = Select {
                    field_list: Animal::FIELDS.iter().map(|x| x.to_string()).collect(),
                    order_list: vec![FieldSortOrder {
                        field: "weight_g".to_string(),
                        order: FieldSortKind::Asc,
                        null_position: None
                    }],
                    ..Default::default()
                };

                let r = DbItem::select(&selection, &mut pool)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|x| AnimalRep::from_item::<&str>(x, None))
                    .collect::<Vec<_>>();
                assert_eq!(r, ret);

                let mut selection = Select::with_fields(Animal::FIELDS)
                    .add_expand_filter(
                        "name",
                        SelectionKind::NotEquals,
                        vec![Value::String("delete from animals".to_string())]
                    );
                selection.order_list = vec![FieldSortOrder {
                        field: "weight_g".to_string(),
                        order: FieldSortKind::Asc,
                        null_position: None
                    }];

                let r = Animal::select(&selection, &mut pool).await.unwrap();
                assert_eq!(r.len(), 3);
                // Tests that the injection attack has truly failed.
                let ret = sqlx::query("select * from animals;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                // Since we still have animals here, the attack did nothing(?).
                assert_eq!(ret.len(), 3);
            },
        )
        .await
    }

    // Deals with some harder selects.
    #[tokio::test]
    async fn test_select_animals2() {
        use crate::db_item::selection::*;
        use crate::Value;

        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g) values('566ff2f3-0078-1eee-89bf-a52f40e61a8d', 'mouse', 50),
                ('566ff2f3-0077-1eee-89bf-a52f40e61a8d', 'rat', 500),
                ('566ff2f3-0080-1eee-89bf-a52f40e61a8d', 'sparrow', 10),
                ('566ff2f3-0090-1eee-89bf-a52f40e61a8d', 'earthworm', 1),
                ('566ff2f3-0100-1eee-89bf-a52f40e61a8d', 'starfish', 200),
                ('566ff2f3-0110-1eee-89bf-a52f40e61a8d', 'salmon', 10000),
                ('566ff2f3-0120-1eee-89bf-a52f40e61a8d', 'dolphin', 500000),
                ('566ff2f3-0130-1eee-89bf-a52f40e61a8d', 'horse', 500000),
                ('566ff2f3-0140-1eee-89bf-a52f40e61a8d', 'guinea pig', 800),
                ('566ff2f3-0150-1eee-89bf-a52f40e61a8d', 'marmot', 2000),
                ('566ff2f3-0160-1eee-89bf-a52f40e61a8d', 'hamster', 50),
                ('566ff2f3-0170-1eee-89bf-a52f40e61a8d', 't-rex', 10000000),
                ('566ff2f3-0180-1eee-89bf-a52f40e61a8d', 'velociraptor', 20000),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a8d', 'capybara', 50000)",
            ),
            |mut pool| async move {
                // Test 1: All animals.
                let selection1 = Select::full::<Animal>();
                // Test 2: Sorted animals
                let selection2 = Select::full::<Animal>()
                    .add_replace_order("name", FieldSortKind::Asc);
                // Test 3: Sorted animals, first.
                let selection3 = Select::full::<Animal>()
                    .take_first()
                    .add_replace_order("weight_g", FieldSortKind::Desc);

                // Test 4: The latest alphabetical animal in the weight range between
                // 10g and 1000g.
                let x = vec![Value::from(50), Value::from(1000)];
                let selection4 = Select::full::<Animal>()
                    .take_first()
                    .add_expand_filter("weight_g", SelectionKind::Between, x)
                    .add_replace_order("name", FieldSortKind::Desc);

                let all_animals = Animal::select(&selection1, &mut pool)
                    .await
                    .unwrap();
                assert_eq!(all_animals.len(), 14);

                let all_animals = Animal::select(&selection2, &mut pool)
                    .await
                    .unwrap();
                assert_eq!(all_animals.len(), 14);

                assert_eq!(&all_animals[0].name, "capybara");
                assert_eq!(&all_animals[13].name, "velociraptor");

                let ret = sqlx::query("SELECT * FROM animals ORDER BY name ASC;")
                    .map(|x| Animal::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(all_animals, ret);

                let first_animal = Animal::select(&selection3, &mut pool)
                    .await
                    .unwrap();
                assert_eq!(first_animal.len(), 1);
                assert_eq!(&first_animal[0].name, "t-rex");
                assert_eq!(first_animal[0].weight_g, 10_000_000);
                assert_eq!(
                    &first_animal[0].id.to_string(),
                    "566ff2f3-0170-1eee-89bf-a52f40e61a8d"
                );

                let first_animal = Animal::select(&selection4, &mut pool)
                    .await
                    .unwrap();
                assert_eq!(first_animal.len(), 1);
                assert_eq!(&first_animal[0].name, "starfish");
                assert_eq!(first_animal[0].weight_g, 200);
                assert_eq!(
                    &first_animal[0].id.to_string(),
                    "566ff2f3-0100-1eee-89bf-a52f40e61a8d"
                );
            }
        )
        .await
    }

    #[tokio::test]
    async fn select_animals_with_null_position() {
        use crate::db_item::selection::*;

        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g, habitat) values
                ('00000000-0000-0000-0000-000000000001', 'mouse', 50, 'Habitat 2'),
                ('00000000-0000-0000-0000-000000000002', 'rat', 500, 'Habitat 1'),
                ('00000000-0000-0000-0000-000000000003', 'capybara', 50000, NULL)",
            ),
            |mut pool| async move {
                // Дефолтное поведение
                {
                    let expected_order = vec![
                        uuid!("00000000-0000-0000-0000-000000000002"),
                        uuid!("00000000-0000-0000-0000-000000000001"),
                        uuid!("00000000-0000-0000-0000-000000000003"),
                    ];

                    let selection = Select::with_fields(Animal::FIELDS)
                        .add_replace_order_asc("habitat");

                    let res = Animal::select(&selection, &mut pool)
                        .await
                        .unwrap();

                    expected_order.into_iter().zip(res).enumerate().for_each(|(idx, (expected, fact))| {
                        assert_eq!(expected, fact.id, "{} элемент не на ожидаемом месте", idx);
                    });
                };

                // Для случая с явной позицией NULL значений
                {
                    let expected_order = vec![
                        uuid!("00000000-0000-0000-0000-000000000003"),
                        uuid!("00000000-0000-0000-0000-000000000002"),
                        uuid!("00000000-0000-0000-0000-000000000001"),
                    ];

                    let selection = Select::with_fields(Animal::FIELDS)
                        .add_replace_order_asc("habitat")
                        .with_nulls_first();

                    let res = Animal::select(&selection, &mut pool)
                        .await
                        .unwrap();

                    expected_order.into_iter().zip(res).enumerate().for_each(|(idx, (expected, fact))| {
                        assert_eq!(expected, fact.id, "{} элемент не на ожидаемом месте", idx);
                    });
                };
            },
        )
        .await
    }

	#[tokio::test]
    async fn test_select_null_animals() {
        use crate::db_item::selection::*;
        use crate::Value;

        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g, habitat) values
                ('566ff2f3-0078-1eee-89bf-a52f40e61a6d', 'mouse', 50, 'Bolshaya Mishka'),
                ('566ff2f3-0077-1eee-89bf-a52f40e61a7d', 'rat', 500, 'Malenkaya Krisa'),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a8d', 'capybara', 50000, NULL),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a9d', 'capybara', 40000, NULL)
                ",
            ),
            |mut pool| async move {
                let selection = Select::full::<Animal>().add_expand_filter(
                  "habitat",
                  SelectionKind::Equals,
                  vec![Value::Null]
                );

                let r = Animal::select(&selection, &mut pool).await.unwrap();
                assert_eq!(r.len(), 2);
                assert!(r.into_iter().all(|animal| {
                  animal.name == "capybara" && animal.habitat == None
                }));
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_complex_select_null_animals() {
        use crate::db_item::selection::*;
        use crate::Value;

        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g, habitat) values
                ('566ff2f3-0078-1eee-89bf-a52f40e61a1d', 'mouse', 50, 'Bolshaya Mishka'),
                ('566ff2f3-0077-1eee-89bf-a52f40e61a2d', 'rat', 500, 'Malenkaya Krisa'),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a3e', 'capybara', 50000, 'Capybaring'),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a4e', 'capybara', 50000, 'FAKE Capybaring'),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a5d', 'capybara', 70000, 'Capybaring'),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a6d', 'capybara', 60000, NULL)
                ",
            ),
            |mut pool| async move {
                let selection = Select::full::<Animal>().add_expand_filter(
                  "habitat",
                  SelectionKind::In,
                  vec![Value::Null, Value::String(String::from("FAKE Capybaring"))]
                );

                let r = Animal::select(&selection, &mut pool).await.unwrap();
                assert_eq!(r.len(), 2);
                assert!(r.into_iter().all(|animal| {
                   animal.name == "capybara" &&
                    (animal.habitat == None || animal.habitat == Some(String::from("FAKE Capybaring")))
                }));
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_more_complex_select_null_animals() {
        use crate::db_item::selection::*;
        use crate::Value;

        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g, habitat) values
                ('566ff2f3-0078-1eee-89bf-a52f40e61a1d', 'mouse', 50, 'Bolshaya Mishka'),
                ('566ff2f3-0077-1eee-89bf-a52f40e61a2d', 'rat', 500, 'Malenkaya Krisa'),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a3e', 'capybara', 50000, 'Capybaring'),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a4e', 'fake-capybara', 50000, 'FAKE Capybaring'),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a5d', 'capybara', 70000, 'Capybaring'),
                ('566ff2f3-0076-1eee-89bf-a52f40e61a6d', 'capybara', 60000, NULL)
                ",
            ),
            |mut pool| async move {
                let selection = Select::full::<Animal>().add_expand_filter(
                  "habitat",
                  SelectionKind::In,
                  vec![Value::Null, Value::String(String::from("FAKE Capybaring"))]
                ).add_expand_filter(
                  "name",
                  SelectionKind::Contains,
                  vec![Value::String(String::from("fake-capybara"))]
                );

                let r = Animal::select(&selection, &mut pool).await.unwrap();
                assert_eq!(r.len(), 1);
                assert_eq!(r[0].id, Uuid::parse_str("566ff2f3-0076-1eee-89bf-a52f40e61a4e").unwrap());

                let selection = Select::full::<Animal>().add_expand_filter(
                  "habitat",
                  SelectionKind::Contains,
                  vec![Value::String(String::from("(?i)capy"))]
                ).add_expand_filter(
                  "name",
                  SelectionKind::Contains,
                  vec![Value::String(String::from("ake-capybar"))]
                );

                let r = Animal::select(&selection, &mut pool).await.unwrap();
                assert_eq!(r.len(), 1);
                assert_eq!(r[0].id, Uuid::parse_str("566ff2f3-0076-1eee-89bf-a52f40e61a4e").unwrap());
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_update_null_animals() {
        use crate::db_item::selection::*;
        use crate::Value;

        run_db_test(
            "animals",
            "(id Uuid NOT NULL, name text NOT NULL, weight_g int NOT NULL, habitat TEXT)",
            Some(
                "(id, name, weight_g, habitat) values
                ('566ff2f3-0076-1eee-89bf-a52f40e61a4e', 'fake-capybara', 50000, 'FAKE Capybaring')
                ",
            ),
            |mut pool| async move {
                let selection = Select::full::<Animal>().add_expand_filter(
                    "habitat",
                    SelectionKind::In,
                    vec![Value::String(String::from("FAKE Capybaring"))]
                );

                let mut r = Animal::select(&selection, &mut pool).await.unwrap();
                assert_eq!(r.len(), 1);

                r[0].habitat = None;
                DbItem::update_vec(
                  &r,
                  Some(&["habitat"]),
                  &mut pool
               ).await.unwrap();

               let new_selection = Select::full::<Animal>().add_expand_filter(
                "habitat",
                SelectionKind::Equals,
                vec![Value::Null]
              );
              let r = Animal::select(&new_selection, &mut pool).await.unwrap();
              assert_eq!(r.len(), 1);
              assert_eq!(r[0].habitat, None);
            },
        )
        .await
    }
}

mod serial_id_table {
    use super::*;

    #[derive(Debug, Clone, DbItem, DbAdaptor)]
    #[adaptor_derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
    #[item_table = "serial_id_table"]
    $(#[$aggr_arrays])?
    struct Data {
        #[item_field_pkey]
        #[item_field_autogen]
        id: i64,
        val: String,
    }

    #[tokio::test]
    async fn test_select_offset_limited() {
        run_db_test(
            Data::TABLE,
            "(id bigserial, val TEXT NOT NULL)",
            Some("(val) VALUES('a'),('d'),('c'),('b'),('f'),('g'),('h'),('k'),('j'),('i'),('z'),('y')"),
            |mut pool| async move {
                let select = Select::default()
                    .add_replace_order_asc("id")
                    .offset(5)
                    .take_n(3);
                let records = Data::select(&select, &mut pool).await.unwrap();

                assert_eq!(records.len(), 3);

                assert_eq!(records[0].id, 6);
                assert_eq!(&records[0].val, "g");
                assert_eq!(records[1].id, 7);
                assert_eq!(&records[1].val, "h");
                assert_eq!(records[2].id, 8);
                assert_eq!(&records[2].val, "k");
        }).await
    }

    #[tokio::test]
    async fn test_insert_data() {
        run_db_test(
            Data::TABLE,
            "(id bigserial, val TEXT NOT NULL)",
            None,
            |mut pool| async move {
                let d_rep = DataRep {
                    id: Some(9999),
                    val: Some("Some important text.".to_string()),
                };
                let expected = DataRep {
                    id: Some(1),
                    val: Some("Some important text.".to_string()),
                };
                let inserted_rows = d_rep
                    .clone()
                    .into_item()
                    .unwrap()
                    .insert(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(inserted_rows, 1);

                let ret = sqlx::query("select * from serial_id_table;")
                    .map(|x| Data::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap()
                    .pop()
                    .unwrap();
                let ret_rep = DataRep::from_item::<&str>(ret, None);
                assert_eq!(ret_rep, expected);
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_update_data() {
        run_db_test(
            Data::TABLE,
            "(id bigserial, val TEXT NOT NULL)",
            Some("(val) values('some important text')"),
            |mut pool| async move {
                let ret = sqlx::query("select * from serial_id_table;")
                    .map(|x| Data::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(ret[0].id, 1);
                assert_eq!(&ret[0].val, "some important text");

                let d_rep = DataRep {
                    id: Some(1),
                    val: Some("Some important text.".to_string()),
                };
                let rows = d_rep
                    .clone()
                    .into_item()
                    .unwrap()
                    .update(None, &mut pool)
                    .await
                    .unwrap();
                assert_eq!(rows, 1);

                let ret = sqlx::query("select * from serial_id_table;")
                    .map(|x| Data::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();
                assert_eq!(ret[0].id, 1);
                assert_eq!(&ret[0].val, "Some important text.");
            },
        )
        .await
    }
  }

mod versioned {
    use super::*;
    use crate::db_item::DbVersioned;
    use sqlx::{Transaction, Postgres};

    #[derive(DbVersioned, DbItem, DbItemExt, PartialEq, Debug, Clone, Copy)]
    #[item_table = "moose"]
    #[db_version_table = "moose_version"]
    struct Moose {
        id: i64,
        #[item_field_pkey]
        uuid: Uuid,
        rank: i32,
    }

    impl Moose {
        fn new(id: i64, rank: i32) -> Self {
            let uuid = Uuid::new_v4();
            Self { id, rank, uuid }
        }
        fn version(&self, pricing_version: i16) -> MooseVersion {
            MooseVersion {
                id: self.id,
                rank: self.rank,
                pricing_version,
                uuid: self.uuid,
            }
        }
    }

    async fn get_mooses(pool: &mut Transaction<'_, Postgres>) -> Vec<MooseVersion> {
        sqlx::query(
            "SElECT * from moose_version ORDER BY pricing_version,id"
        )
        .try_map(|r| MooseVersion::from_row(&r))
        .fetch_all(pool)
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_moose_conflict_ids() {
        run_db_test(
            Moose::TABLE,
            "(id bigint NOT NULL, uuid uuid NOT NULL PRIMARY KEY, rank integer)",
            None,
            |mut pool| async move {
                sqlx::query("DROP TABLE IF EXISTS moose_version")
                    .execute(&mut pool)
                    .await
                    .unwrap();
                sqlx::query("CREATE TABLE moose_version(
                    id bigint NOT NULL,
                    uuid uuid NOT NULL,
                    pricing_version SMALLINT NOT NULL,
                    rank integer);
                ")
                    .execute(&mut pool)
                    .await
                    .unwrap();

                let mut m1 = Moose::new(1, 99)
                    .insert_returning(&mut pool)
                    .await
                    .expect("Could not insert m1");
                let m2 = Moose::new(1, 98)
                    .insert_returning(&mut pool)
                    .await
                    .expect("Could not insert m2");
                let m3 = Moose::new(1, 97)
                    .insert_returning(&mut pool)
                    .await
                    .expect("Could not insert m3");

                let to_version = [m1.clone(), m2.clone(), m3.clone()];
                let mut versions1 = Moose::insert_version_vec_returning(&to_version, &mut pool)
                    .await
                    .unwrap();

                let versions = get_mooses(&mut pool).await;
                assert_eq!(versions1, versions, "first assert");

                let m1b = Moose::new(1, 99);
                let m2b = Moose::new(2, 98);
                let m3b = Moose::new(3, 34);
                let mut new_mooses = vec![m1b, m2b, m3b];

                Moose::insert_vec(&mut new_mooses, &mut pool).await.unwrap();

                let mut versions2 = Moose::insert_version_vec_returning(
                    &new_mooses,
                    &mut pool
                )
                .await
                .unwrap();

                versions2.append(&mut versions1);
                versions2.sort_unstable_by(|a, b| {
                    match a.pricing_version.cmp(&b.pricing_version) {
                        std::cmp::Ordering::Equal => a.id.cmp(&b.id),
                        x => x,
                    }
                });
                let versions = get_mooses(&mut pool).await;

                assert_eq!(versions2, versions, "second assert");

                let mut exp = vec![
                    m1.version(1),
                    new_mooses[1].version(1),
                    new_mooses[2].version(1),
                    m2.version(2),
                    m3.version(3),
                    new_mooses[0].version(4),
                ];
                assert_eq!(exp, versions, "third assert");

                // Moose number one won big!
                m1.rank = 1;
                let m1_updated: Moose = m1
                    .update_returning::<_, &str>(None, None, &mut pool)
                    .await
                    .unwrap();

                assert_eq!(m1, m1_updated);
                let m1v5 = Moose::insert_version_vec_returning(
                    &[m1_updated],
                    &mut pool
                )
                .await
                .unwrap()
                .pop()
                .unwrap();
                let m1v5_exp = m1.version(5);
                assert_eq!(m1v5, m1v5_exp);

                let versions = get_mooses(&mut pool).await;

                exp.push(m1v5_exp);
                assert_eq!(exp, versions, "fourth assert");
            }
        )
        .await
    }

#[derive(DbVersioned, DbItem, DbItemExt, PartialEq, Debug, Clone)]
#[item_table = "complex"]
#[db_version_table = "complex_version"]
struct ComplexExample {
    id: i64,
    #[item_field_pkey]
    uuid: Uuid,
    rank: i32,
    optional_note: Option<String>,
    x: i64,
    y: i64,
}

impl ComplexExample {
    fn new(id: i64, rank: i32, x: i64, y: i64) -> Self {
        let uuid = Uuid::new_v4();
        let optional_note = None;

        Self { id, uuid, rank, x, y, optional_note }
    }
}

async fn get_complex_version(pool: &mut Transaction<'_, Postgres>) -> Vec<ComplexExampleVersion> {
    sqlx::query(
        "SElECT * from complex_version ORDER BY pricing_version,id"
    )
    .try_map(|r| ComplexExampleVersion::from_row(&r))
    .fetch_all(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn test_all_version_fields_are_inserted() {
    // This is in fact a tiny bit wrong, since a version should in theory exist for these two records,
    // but will not, since we are inserting them directly.
    //
    // However, the point of the test is to see if all fields of the original
    // record are saved in the version, so I suppose this can be forgiven.
    // It is also tested in other tests.
    run_db_test(
        ComplexExample::TABLE,
        "(id bigint NOT NULL, uuid uuid NOT NULL PRIMARY KEY, rank integer, optional_note TEXT, x BIGINT, y BIGINT)",
        Some("
            (id, uuid, rank, optional_note, x, y) values
            (1,'566ff2f3-0000-0000-89bf-a52f40e61a1d',99,'not a winner, but not bad',39000,54000),
            (2,'566ff2f3-0000-1111-89bf-a52f40e61a1d',151,'hopeless',234,51000)
        "),
        |mut pool| async move {
            sqlx::query("CREATE TABLE complex_version(
                id bigint NOT NULL,
                uuid uuid NOT NULL PRIMARY KEY,
                rank integer,
                optional_note TEXT,
                x BIGINT,
                y BIGINT,
                pricing_version SMALLINT NOT NULL
            )")
            .execute(&mut pool)
            .await
            .expect("Cannot create version table.");

            let mut complex = ComplexExample::new(1, 84, 0, 0);
            complex.uuid = crate::uuid!("566ff2f3-0000-0000-89bf-a52f40e61a1d");

            let ret = complex
                .update_returning::<_, &str>(Some(&["rank"]), None, &mut pool)
                .await
                .unwrap();

            let mut exp = ComplexExample::new(1, 84, 39000, 54000);
            exp.optional_note = Some("not a winner, but not bad".to_string());
            exp.uuid = crate::uuid!("566ff2f3-0000-0000-89bf-a52f40e61a1d");

            assert_eq!(ret, exp, "updated record does not match expected.");

            // Use explicit code here just in case.
            let exp_version = vec![ComplexExampleVersion {
                id: 1,
                pricing_version: 1,
                uuid: crate::uuid!("566ff2f3-0000-0000-89bf-a52f40e61a1d"),
                rank: 84,
                optional_note: Some("not a winner, but not bad".to_string()),
                x: 39000,
                y: 54000,
            }];
            let version1 = ComplexExample::insert_version_vec_returning(
                &[ret],
                &mut pool,
            )
            .await
            .unwrap();

            let version = get_complex_version(&mut pool).await;

            assert_eq!(version1, exp_version, "version record does not match expected.");
            assert_eq!(version, exp_version, "version record does not match expected.");
    }).await
}

}

mod renamed_fields {
    use super::*;
    use crate::Value;

    #[derive(DbItem, DbAdaptor, PartialEq, Debug, Clone)]
    #[adaptor_derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
    #[item_table = "strictly_regulated_table_of_the_price_moose"]
    struct Moose {
        #[item_field_pkey]
        #[db_field_name = "strictly_regulated_id_of_the_price_moose"]
        id: i64,
        #[db_field_name = "strictly_regulated_price_analysis_rank_of_the_price_moose"]
        rank: i32,
    }

    impl Moose {
        fn new(id: i64, rank: i32) -> Self {
            Self { id, rank }
        }
    }

    #[tokio::test]
    async fn test_insert_update_fetch_data() {
        run_db_test(
            Moose::TABLE,
            "(strictly_regulated_id_of_the_price_moose bigserial, strictly_regulated_price_analysis_rank_of_the_price_moose INTEGER)",
            None,
            |mut pool| async move {
                assert_eq!(Moose::id, "strictly_regulated_id_of_the_price_moose");
                assert_eq!(Moose::rank, "strictly_regulated_price_analysis_rank_of_the_price_moose");

                let mut m1 = Moose::new(1, 88);
                let m2 = Moose::new(2, 76);
                let m3 = Moose::new(3, 96);
                let m4 = Moose::new(4, 2);

                let mut mooses = vec![m2, m3, m4];

                let m1r = m1.insert_returning(&mut pool).await.unwrap();
                let mooses_r = Moose::insert_vec_returning(&mut mooses, &mut pool)
                    .await
                    .unwrap();

                assert_eq!(mooses, mooses_r);
                assert_eq!(m1, m1r);

                let m2 = Moose::new(2, 2);
                let m4 = Moose::new(4, 3);
                let m2r = m2.update_returning::<_, &str>(None, None, &mut pool)
                    .await
                    .unwrap();
                let m4r = m4.update_returning::<_, &str>(None, None, &mut pool)
                    .await
                    .unwrap();

                assert_eq!(m2, m2r);
                assert_eq!(m4, m4r);

                let ids = [1, 2, 3, 4].iter().map(Value::from);
                let select = Select::full::<Moose>()
                    .in_any("strictly_regulated_id_of_the_price_moose", ids)
                    .add_replace_order_asc(Moose::id);

                let all = Moose::select(&select, &mut pool).await.unwrap();

                assert_eq!(
                    all,
                    vec![Moose::new(1,88),Moose::new(2,2),Moose::new(3,96),Moose::new(4,3)]
                )
            },
        )
        .await
    }

}

mod delete {
    use super::*;
    use crate::db_item::DbItemDel;
    use crate::db_item::selection::{Filter, FilterTree};

    #[derive(DbItem, PartialEq, Debug, Clone)]
    #[item_table = "moose"]
    struct Moose {
        #[item_field_pkey]
        id: i64,
        rank: i32,
    }

    impl DbItemDel for Moose {}

    impl Moose {
        fn new(id: i64, rank: i32) -> Self {
            Self { id, rank }
        }
    }
    #[tokio::test]
    async fn test_delete() {
        run_db_test(
            Moose::TABLE,
            "(id bigserial, rank INTEGER)",
            None,
            |mut pool| async move {
                assert_eq!(Moose::id, "id");
                assert_eq!(Moose::rank, "rank");

                let m1 = Moose::new(1, 88);
                let m2 = Moose::new(2, 76);
                let m3 = Moose::new(3, 96);
                let m4 = Moose::new(4, 2);

                let mut mooses = vec![m1, m2, m3, m4];

                let mooses_r = Moose::insert_vec_returning(&mut mooses, &mut pool)
                    .await
                    .unwrap();
                assert_eq!(mooses_r.len(), 4);

                let ids = [2, 3];
                let rank = [75, 97];
                let filter1 = Filter::in_any(Moose::id, ids);
                let filter2 = Filter::between(Moose::rank, rank[0], rank[1]);

                let filters = FilterTree::and_from_list([filter1, filter2]);

                let m = Moose::delete_returning(&filters, &mut pool).await.unwrap();

                assert_eq!(m.len(), 2);
                assert!(m.iter().any(|m| m.id == 2 && m.rank == 76));
                assert!(m.iter().any(|m| m.id == 3 && m.rank == 96));

                let select = Select::full::<Moose>()
                    .add_replace_order_asc(Moose::id);
                let all = Moose::select(&select, &mut pool).await.unwrap();
                assert_eq!(all.len(), 2);
                assert_eq!(all, vec![Moose::new(1,88),Moose::new(4,2)]);
            },
        )
        .await
    }
}

mod autogen_key {

    use super::*;
    use crate::db_item::DbItem;

    #[derive(DbItem, DbAdaptor, PartialEq, Debug, Clone)]
    #[adaptor_derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
    #[item_table = "foo"]
    $(#[$aggr_arrays])?
    struct Foo {
        #[item_field_pkey]
        #[item_field_autogen]
        id: i16,
        base: i32,
        #[item_field_autogen_always]
        always_autogen: i32,
    }

    const CREATE_FOO: &str = r#"(
id SMALLINT PRIMARY KEY GENERATED BY DEFAULT AS IDENTITY,
base INTEGER,
always_autogen INTEGER GENERATED ALWAYS AS (base + 20) STORED
)"#;

    const VALUES_FOO: &str = r#"(id, base) VALUES(1, 20)"#;

    #[tokio::test]
    async fn insert() {
        run_db_test(Foo::TABLE, CREATE_FOO, None, |mut pool| async move {
            let mut item = FooRep::default();
            item.base = 20.into();

            let c =
                item.clone().into_item().unwrap().insert(&mut pool).await.unwrap();
            assert_eq!(c, 1);

            let items = sqlx::query("select * from foo")
                .map(|x| Foo::from_row(&x).unwrap())
                .fetch_all(&mut pool)
                .await
                .unwrap();

            let item_exp = Foo {
                id: 1,
                base: 20,
                always_autogen: 40,
            };

            assert_eq!(items.len(), 1, "incorrect update length");
            assert_eq!(items[0], item_exp, "incorrect update content");
        })
        .await
    }

    #[tokio::test]
    async fn insert_autogen() {
        run_db_test(Foo::TABLE, CREATE_FOO, None, |mut pool| async move {
            let mut item = FooRep::default();
            item.base = 20.into();
            item.always_autogen = 200.into();

            let c =
                item.clone().into_item().unwrap().insert(&mut pool).await.unwrap();
            assert_eq!(c, 1);

            let items = sqlx::query("select * from foo")
                .map(|x| Foo::from_row(&x).unwrap())
                .fetch_all(&mut pool)
                .await
                .unwrap();

            let item_exp = Foo {
                id: 1,
                base: 20,
                always_autogen: 40,
            };

            assert_eq!(items.len(), 1, "incorrect update length");
            assert_eq!(items[0], item_exp, "incorrect update content");
        })
        .await
    }

    #[tokio::test]
    async fn update_fields() {
        run_db_test(
            Foo::TABLE,
            CREATE_FOO,
            Some(VALUES_FOO),
            |mut pool| async move {
                let id = 1;
                let mut item = FooRep::default();
                item.id = Some(id);
                item.base = Some(30);
                item.always_autogen = Some(100);

                let c = item
                    .clone()
                    .into_item()
                    .unwrap()
                    .update(Some(&[Foo::base, Foo::always_autogen]), &mut pool)
                    .await
                    .unwrap();
                assert_eq!(c, 1);

                let items = sqlx::query("select * from foo")
                    .map(|x| Foo::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                let item_exp = Foo {
                    id,
                    base: 30,
                    always_autogen: 50,
                };

                assert_eq!(items.len(), 1, "incorrect update length");
                assert_eq!(items[0], item_exp, "incorrect update content");
            },
        )
        .await
    }

    #[tokio::test]
    async fn update_fields_vec() {
        run_db_test(
            Foo::TABLE,
            CREATE_FOO,
            Some(VALUES_FOO),
            |mut pool| async move {
                let id = 1;
                let mut item = FooRep::default();
                item.id = Some(id);
                item.base = Some(30);
                item.always_autogen = Some(100);

                let c = item.clone().into_item().unwrap();

                let c = Foo::update_vec(
                    &[c],
                    Some(&[Foo::base, Foo::always_autogen]),
                    &mut pool,
                )
                .await
                .unwrap();
                assert_eq!(c, 1);

                let items = sqlx::query("select * from foo")
                    .map(|x| Foo::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                let item_exp = Foo {
                    id,
                    base: 30,
                    always_autogen: 50,
                };

                assert_eq!(items.len(), 1, "incorrect update length");
                assert_eq!(items[0], item_exp, "incorrect update content");
            },
        )
        .await
    }

    #[tokio::test]
    async fn update_some_fields() {
        run_db_test(
            Foo::TABLE,
            CREATE_FOO,
            Some(VALUES_FOO),
            |mut pool| async move {
                let id = 1;
                let mut item = FooRep::default();
                item.id = Some(id);
                item.base = Some(30);
                item.always_autogen = Some(100);

                let c = item
                    .clone()
                    .into_item()
                    .unwrap()
                    .update(Some(&[Foo::base]), &mut pool)
                    .await
                    .unwrap();
                assert_eq!(c, 1);

                let items = sqlx::query("select * from foo")
                    .map(|x| Foo::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                let item_exp = Foo {
                    id,
                    base: 30,
                    always_autogen: 50,
                };

                assert_eq!(items.len(), 1, "incorrect update length");
                assert_eq!(items[0], item_exp, "incorrect update content");
            },
        )
        .await
    }
}

mod autogen_fields {

    use super::*;
    use crate::db_item::DbItem;

    #[derive(DbItem, DbAdaptor, PartialEq, Debug, Clone)]
    #[adaptor_derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
    #[item_table = "bar"]
    $(#[$aggr_arrays])?
    struct Bar {
        #[item_field_pkey]
        id: i16,
        base: i32,
        #[item_field_autogen]
        autogen: i32,
        #[item_field_autogen_always]
        always_autogen: i32,
        s: String,
    }

    const CREATE_BAR: &str = r#"(
id SMALLINT PRIMARY KEY,
base INTEGER,
autogen INTEGER GENERATED BY DEFAULT AS IDENTITY,
always_autogen INTEGER GENERATED ALWAYS AS (base + 20) STORED,
s VARCHAR
)"#;

    const VALUES_BAR: &str = r#"(id, base) VALUES(1, 20)"#;

    #[tokio::test]
    async fn insert() {
        run_db_test(Bar::TABLE, CREATE_BAR, None, |mut pool| async move {
            let mut item = BarRep::default();
            item.id = 1.into();
            item.base = 20.into();

            let c =
                item.clone().into_item().unwrap().insert(&mut pool).await.unwrap();
            assert_eq!(c, 1);

            let items = sqlx::query("select * from bar")
                .map(|x| Bar::from_row(&x).unwrap())
                .fetch_all(&mut pool)
                .await
                .unwrap();

            let item_exp = Bar {
                id: 1,
                base: 20,
                autogen: 1,
                always_autogen: 40,
                s: String::default(),
            };

            assert_eq!(items.len(), 1, "incorrect update length");
            assert_eq!(items[0], item_exp, "incorrect update content");
        })
        .await
    }

    #[tokio::test]
    async fn insert_autogen() {
        run_db_test(Bar::TABLE, CREATE_BAR, None, |mut pool| async move {
            let mut item = BarRep::default();
            item.id = 1.into();
            item.base = 20.into();
            item.autogen = 100.into();
            item.always_autogen = 200.into();

            let c =
                item.clone().into_item().unwrap().insert(&mut pool).await.unwrap();
            assert_eq!(c, 1);

            let items = sqlx::query("select * from bar")
                .map(|x| Bar::from_row(&x).unwrap())
                .fetch_all(&mut pool)
                .await
                .unwrap();

            let item_exp = Bar {
                id: 1,
                base: 20,
                autogen: 1,
                always_autogen: 40,
                s: String::default(),
            };

            assert_eq!(items.len(), 1, "incorrect update length");
            assert_eq!(items[0], item_exp, "incorrect update content");
        })
        .await
    }

    #[tokio::test]
    async fn update() {
        run_db_test(
            Bar::TABLE,
            CREATE_BAR,
            Some(VALUES_BAR),
            |mut pool| async move {
                let id = 1;
                let s = "foo".to_owned();
                let mut item = BarRep::default();
                item.id = Some(id);
                item.base = Some(30);
                item.autogen = Some(10);
                item.always_autogen = Some(100);
                item.s = Some(s.clone());

                let c = item
                    .clone()
                    .into_item()
                    .unwrap()
                    .update(None, &mut pool)
                    .await
                    .unwrap();
                assert_eq!(c, 1);

                let items = sqlx::query("select * from bar")
                    .map(|x| Bar::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                let item_exp = Bar {
                    id,
                    base: 30,
                    autogen: 10,
                    always_autogen: 50,
                    s,
                };

                assert_eq!(items.len(), 1, "incorrect update length");
                assert_eq!(items[0], item_exp, "incorrect update content");
            },
        )
        .await
    }

    #[tokio::test]
    async fn update_fields() {
        run_db_test(
            Bar::TABLE,
            CREATE_BAR,
            Some(VALUES_BAR),
            |mut pool| async move {
                let id = 1;
                let s = "foo".to_owned();
                let mut item = BarRep::default();
                item.id = Some(id);
                item.base = Some(30);
                item.autogen = Some(10);
                item.always_autogen = Some(100);
                item.s = Some(s.clone());

                let c = item
                    .clone()
                    .into_item()
                    .unwrap()
                    .update(
                        Some(&[
                            Bar::base,
                            Bar::autogen,
                            Bar::always_autogen,
                            Bar::s,
                        ]),
                        &mut pool,
                    )
                    .await
                    .unwrap();
                assert_eq!(c, 1);

                let items = sqlx::query("select * from bar")
                    .map(|x| Bar::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                let item_exp = Bar {
                    id,
                    base: 30,
                    autogen: 10,
                    always_autogen: 50,
                    s,
                };

                assert_eq!(items.len(), 1, "incorrect update length");
                assert_eq!(items[0], item_exp, "incorrect update content");
            },
        )
        .await
    }

    #[tokio::test]
    async fn update_ret() {
        run_db_test(
            Bar::TABLE,
            CREATE_BAR,
            Some(VALUES_BAR),
            |mut pool| async move {
                let id = 1;
                let s = "foo".to_owned();
                let mut item = BarRep::default();
                item.id = Some(id);
                item.base = Some(30);
                item.autogen = Some(10);
                // THis field has an illegal value. On update it will be
                // base+20=30+20=50
                item.always_autogen = Some(100);
                item.s = Some(s.clone());

                let db_item = item.clone().into_item().unwrap();

                let c_vec = item
                    .clone()
                    .into_item()
                    .unwrap()
                    .update_returning::<_, &str>(None, None, &mut pool)
                    .await
                    .unwrap();

                assert_ne!(c_vec, db_item);
                let mut cor_item = db_item.clone();

                cor_item.always_autogen = 50;
                assert_ne!(c_vec, db_item);

                let items = sqlx::query("select * from bar")
                    .map(|x| Bar::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                let item_exp = Bar {
                    id,
                    base: 30,
                    autogen: 10,
                    always_autogen: 50,
                    s,
                };

                assert_eq!(items.len(), 1, "incorrect update length");
                assert_eq!(items[0], item_exp, "incorrect update content");
            },
        )
        .await
    }

    #[tokio::test]
    async fn update_vec_ret() {
        run_db_test(
            Bar::TABLE,
            CREATE_BAR,
            Some(VALUES_BAR),
            |mut pool| async move {
                let id = 1;
                let s = "foo".to_owned();
                let mut item = BarRep::default();
                item.id = Some(id);
                item.base = Some(30);
                // THis field has an illegal value. On update it will be
                // base+20=30+20=50
                item.always_autogen = Some(100);
                item.s = Some(s.clone());

                let db_item = item.clone().into_item().unwrap();

                let c_vec = Bar::update_vec_returning::<&str>(
                    &[db_item.clone()],
                    None,
                    None,
                    &mut pool,
                )
                .await
                .unwrap()
                .pop()
                .unwrap();

                assert_ne!(c_vec, db_item);
                let mut cor_item = db_item.clone();

                cor_item.always_autogen = 50;
                assert_ne!(c_vec, db_item);

                let items = sqlx::query("select * from bar")
                    .map(|x| Bar::from_row(&x).unwrap())
                    .fetch_all(&mut pool)
                    .await
                    .unwrap();

                let item_exp = Bar {
                    id,
                    base: 30,
                    autogen: 0,
                    always_autogen: 50,
                    s,
                };

                assert_eq!(items.len(), 1, "incorrect update length");
                assert_eq!(items[0], item_exp, "incorrect update content");
            },
        )
        .await
    }

    #[tokio::test]
    async fn update_vec_ret_n() {
        run_db_test(Bar::TABLE, CREATE_BAR, None, |mut pool| async move {
            const N: i16 = 5;

            let mut items = (0..N)
                .map(|id| {
                    BarRep {
                        id: Some(id),
                        base: Some((id * 10).into()),
                        s: Some(format!("f#{id}")),
                        ..Default::default()
                    }
                    .into_item()
                    .unwrap()
                })
                .collect::<Vec<_>>();

            let c = Bar::insert_vec(items.as_mut(), &mut pool).await.unwrap();
            assert_eq!(c, N as u64);

            for i in 0..(N as usize) {
                items[i].base = (i + 10) as i32;
                items[i].autogen = (i * 10) as i32;
                items[i].always_autogen = (i * 20) as i32;
            }

            let id = 1;
            let s = "foo".to_owned();
            let mut item = BarRep::default();
            item.id = Some(id);
            item.base = Some(30);
            // THis field has an illegal value. On update it will be
            // base+20=30+20=50
            item.always_autogen = Some(100);
            item.s = Some(s.clone());

            let c_vec = Bar::update_vec_returning::<&str>(
                items.as_ref(),
                None,
                None,
                &mut pool,
            )
            .await
            .unwrap();

            assert_eq!(c_vec.len(), N as usize);

            for i in 0..(N as usize) {
                assert_eq!(c_vec[i].base, (i + 10) as i32);
                assert_eq!(c_vec[i].autogen, (i * 10) as i32);
                assert_eq!(c_vec[i].always_autogen, (i + 10 + 20) as i32);
            }

            let items = sqlx::query("select * from bar")
                .map(|x| Bar::from_row(&x).unwrap())
                .fetch_all(&mut pool)
                .await
                .unwrap();

            for i in 0..(N as usize) {
                assert_eq!(c_vec[i].base, (i + 10) as i32);
                assert_eq!(c_vec[i].autogen, (i * 10) as i32);
                assert_eq!(c_vec[i].always_autogen, (i + 10 + 20) as i32);
            }

            assert_eq!(items.len(), N as usize);
        })
        .await
    }
}


mod upsert {
    use super::*;
    use crate::db_item::{DbItem, DbUpsert};

    #[derive(DbItem, PartialEq, Debug, Clone, Copy)]
    #[item_table = "moose"]
    $(#[$aggr_arrays])?
    struct Moose {
        #[item_field_pkey]
        uuid: Uuid,
        weight: i64,
        rank: i32,
    }
    impl DbUpsert for Moose {}

    impl Moose {
        fn new(id: &str, rank: i32, weight: i64) -> Self {
            let uuid = Uuid::parse_str(id).unwrap();
            Self { uuid, rank, weight }
        }
    }

    #[tokio::test]
    async fn test_insert_on_conflict_update() {
        run_db_test(
            Moose::TABLE,
            "(uuid uuid PRIMARY KEY, rank INTEGER, weight BIGINT)",
            None,
            |mut pool| async move {
                assert_eq!(Moose::uuid, "uuid");
                assert_eq!(Moose::rank, "rank");
                assert_eq!(Moose::weight, "weight");

                let m1 =
                    Moose::new("00000000-0000-0000-0000-000000000001", 95, 899);
                let m2 =
                    Moose::new("00000000-0000-0000-0000-000000000002", 76, 899);

                let mut mooses = vec![m1, m2];

                let returned =
                    Moose::upsert_returning(&mut mooses, None, &mut pool)
                        .await
                        .unwrap();

                assert_eq!(returned, mooses, "returned items do not equal input");

                let check =
                    Moose::select(&Default::default(), &mut pool).await.unwrap();
                assert_eq!(check.len(), 2);

                let m1 =
                    Moose::new("00000000-0000-0000-0000-000000000001", 55, 899);
                let m3 =
                    Moose::new("00000000-0000-0000-0000-000000000003", 96, 700);
                let m4 = Moose::new("00000000-0000-0000-0000-000000000004", 2, 345);

                let mut mooses2 = [m1, m2, m3, m4];

                let returned =
                    Moose::upsert_returning(&mut mooses2, None, &mut pool)
                        .await
                        .unwrap();

                let check =
                    Moose::select(&Default::default(), &mut pool).await.unwrap();
                assert_eq!(check.len(), 4);

                assert_eq!(
                    returned, mooses2,
                    "returned items not same as expected"
                );
                assert_ne!(
                    mooses, mooses2,
                    "initial items same as expected, but should not be"
                );
            },
        )
        .await
    }
}

mod upsert_autogen {
    use super::*;
    use crate::db_item::{DbItem, DbUpsert};

    #[derive(DbItem, PartialEq, Debug, Clone, Copy)]
    #[item_table = "moose"]
    $(#[$aggr_arrays])?
    struct Moose {
        #[item_field_pkey]
        uuid: Uuid,
        weight: i64,
        #[item_field_autogen_always]
        auto_rank: i32,
    }
    impl DbUpsert for Moose {}

    impl Moose {
        fn new(id: &str, auto_rank: i32, weight: i64) -> Self {
            let uuid = Uuid::parse_str(id).unwrap();
            Self {
                uuid,
                auto_rank,
                weight,
            }
        }
    }

    #[tokio::test]
    async fn test_insert_on_conflict_update() {
        run_db_test(
            Moose::TABLE,
            "(uuid uuid PRIMARY KEY, auto_rank INTEGER generated always as (weight::INTEGER - 100) STORED, weight BIGINT)",
            None,
            |mut pool| async move {
                assert_eq!(Moose::uuid, "uuid");
                assert_eq!(Moose::auto_rank, "auto_rank");
                assert_eq!(Moose::weight, "weight");

                let m1 =
                    Moose::new("00000000-0000-0000-0000-000000000001", 95, 899);
                let m2 =
                    Moose::new("00000000-0000-0000-0000-000000000002", 76, 899);

                let mut mooses = vec![m1, m2];

                let returned =
                    Moose::upsert_returning(&mut mooses, None, &mut pool)
                        .await
                        .unwrap();

                let expected = vec![
                    Moose::new("00000000-0000-0000-0000-000000000001", 799, 899),
                    Moose::new("00000000-0000-0000-0000-000000000002", 799, 899),
                ];

                assert_eq!(returned, expected, "returned items do not equal input");

                let check =
                    Moose::select(&Default::default(), &mut pool).await.unwrap();
                assert_eq!(check.len(), 2);

                let m1 =
                    Moose::new("00000000-0000-0000-0000-000000000001", 55, 600);
                let m3 =
                    Moose::new("00000000-0000-0000-0000-000000000003", 96, 700);
                let m4 = Moose::new("00000000-0000-0000-0000-000000000004", 2, 345);

                let mut mooses2 = [m1, m2, m3, m4];

                let returned =
                    Moose::upsert_returning(&mut mooses2, None, &mut pool)
                        .await
                        .unwrap();

                let expected = vec![
                    Moose::new("00000000-0000-0000-0000-000000000001", 500, 600),
                    Moose::new("00000000-0000-0000-0000-000000000002", 799, 899),
                    Moose::new("00000000-0000-0000-0000-000000000003", 600, 700),
                    Moose::new("00000000-0000-0000-0000-000000000004", 245, 345)

                ];

                let check =
                    Moose::select(&Default::default(), &mut pool).await.unwrap();

                assert_eq!(check.len(), 4);
                assert_eq!(returned, expected, "returned items not same as expected");
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_insert_on_conflict_update_fields() {
        run_db_test(
            Moose::TABLE,
            "(uuid uuid PRIMARY KEY, auto_rank INTEGER generated always as (weight::INTEGER - 100) STORED, weight BIGINT)",
            None,
            |mut pool| async move {
                assert_eq!(Moose::uuid, "uuid");
                assert_eq!(Moose::auto_rank, "auto_rank");
                assert_eq!(Moose::weight, "weight");

                let m1 =
                    Moose::new("00000000-0000-0000-0000-000000000001", 95, 899);
                let m2 =
                    Moose::new("00000000-0000-0000-0000-000000000002", 76, 899);

                let mut mooses = vec![m1, m2];

                let returned =
                    Moose::upsert_returning(
                            &mut mooses,
                            Some(&[Moose::auto_rank]),
                            &mut pool
                        )
                        .await
                        .unwrap();

                let expected = vec![
                    Moose::new("00000000-0000-0000-0000-000000000001", 799, 899),
                    Moose::new("00000000-0000-0000-0000-000000000002", 799, 899),
                ];

                assert_eq!(returned, expected);

                let check =
                    Moose::select(&Default::default(), &mut pool).await.unwrap();
                assert_eq!(check.len(), 2);

                let m1 =
                    Moose::new("00000000-0000-0000-0000-000000000001", 55, 600);
                let m3 =
                    Moose::new("00000000-0000-0000-0000-000000000003", 96, 700);
                let m4 = Moose::new("00000000-0000-0000-0000-000000000004", 2, 345);

                let mut mooses2 = [m1, m2, m3, m4];
                // Nothing will be updated, since we only update only an unupdatable field.
                let returned =
                    Moose::upsert_returning(
                            &mut mooses2,
                            Some(&[Moose::auto_rank]),
                            &mut pool
                        )
                        .await
                        .unwrap();

                let expected = vec![
                    Moose::new("00000000-0000-0000-0000-000000000001", 799, 899),
                    Moose::new("00000000-0000-0000-0000-000000000002", 799, 899),
                    Moose::new("00000000-0000-0000-0000-000000000003", 600, 700),
                    Moose::new("00000000-0000-0000-0000-000000000004", 245, 345)

                ];

                let check =
                    Moose::select(&Default::default(), &mut pool).await.unwrap();

                assert_eq!(check.len(), 4);
                assert_eq!(returned, expected);
            },
        )
        .await
    }
}

mod recursive {

    use itertools::Itertools;
    use super::*;
    use crate::db_item::DbItem;

    #[derive(DbItem, PartialEq, Debug, Clone)]
    #[item_table = "tree"]
    $(#[$aggr_arrays])?
    struct Tree {
        #[item_field_pkey]
        id: i32,
        parent: Option<i32>,
        name: String,
    }

    const CREATE_TREE: &str = r#"(
id integer PRIMARY KEY,
parent integer,
name varchar
)"#;

    const VALUES_TREE: &str = r#"(id, parent, name) VALUES
(1, null, 'root'),
 (2, 1, 'node1'),
  (3, 2, 'node2'),
  (4, 2, 'node3'),
 (5, 1, 'node4'),
  (6, 5, 'node5'),
  (7, 5, 'node6'),
 (8, 1, 'node7'),
  (9, 8, 'node8'),
   (10, 9, 'node9'),
   (11, 9, 'node10'),
   (12, 9, 'node11'),
   (13, 9, 'node12'),
   (14, 9, 'node13'),
   (15, 9, 'node14')
"#;

    const VALUES_CYCLE_TREE: &str = r#"(id, parent, name) VALUES
(1, null, 'root'),
 (2, 3, 'node1'),
 (3, 2, 'node2')
"#;

    #[tokio::test]
    async fn children() {
        run_db_test(
            Tree::TABLE,
            CREATE_TREE,
            Some(VALUES_TREE),
            |mut pool| async move {
                let s1 = Select::full::<Tree>().eq(Tree::name, "node1");
                let s2 = Select::full::<Tree>();
                let items = Tree::select_recursive(&s1, Tree::id, Tree::parent, &s2, &mut pool).await.unwrap();
                let ids = items.into_iter().map(|x| x.id).sorted().collect::<Vec<_>>();
                assert_eq!(&ids, &[2, 3, 4]);
            }).await
    }

    #[tokio::test]
    async fn children_ids_in() {
        run_db_test(
            Tree::TABLE,
            CREATE_TREE,
            Some(VALUES_TREE),
            |mut pool| async move {
                let s1 = Select::full::<Tree>().eq(Tree::name, "node7");
                let s2 = Select::full::<Tree>().in_any(Tree::id, [9, 11, 13, 15]);
                let items = Tree::select_recursive(&s1, Tree::id, Tree::parent, &s2, &mut pool).await.unwrap();
                let ids = items.into_iter().map(|x| x.id).sorted().collect::<Vec<_>>();
                assert_eq!(&ids, &[9, 11, 13, 15]);
            }).await
    }

    #[tokio::test]
    async fn parents() {
        run_db_test(
            Tree::TABLE,
            CREATE_TREE,
            Some(VALUES_TREE),
            |mut pool| async move {
                let s1 = Select::full::<Tree>().eq(Tree::name, "node14");
                let s2 = Select::full::<Tree>();
                let items = Tree::select_recursive(&s1, Tree::parent, Tree::id, &s2, &mut pool).await.unwrap();
                let ids = items.into_iter().map(|x| x.id).sorted().collect::<Vec<_>>();
                assert_eq!(&ids, &[1, 8, 9, 15]);
            }).await
    }

    #[tokio::test]
    async fn cycle() {
        run_db_test(
            Tree::TABLE,
            CREATE_TREE,
            Some(VALUES_CYCLE_TREE),
            |mut pool| async move {
                let s1 = Select::full::<Tree>().eq(Tree::name, "node1");
                let s2 = Select::full::<Tree>().add_replace_order_asc(Tree::id).distinct_on(&[Tree::id]);
                let items = Tree::select_recursive(&s1, Tree::parent, Tree::id, &s2, &mut pool).await.unwrap();
                let ids = items.into_iter().map(|x| x.id).sorted().collect::<Vec<_>>();
                assert_eq!(&ids, &[2, 3]);
            }).await
    }

}

mod paginated {
    use super::*;
    use crate::db_item::DbItem;

    #[derive(DbItem, Debug, Clone, PartialEq, Eq, Hash)]
    #[item_table = "tbl"]
    $(#[$aggr_arrays])?
    struct Tbl {
        #[item_field_pkey]
        id: i32,
        name: String,
    }

    const CREATE: &str = r#"(
id serial primary key,
name varchar
)"#;

    const VALUES: &str = r#"(name) VALUES
('Alfa'), ('November'),
('Bravo'), ('Oscar'),
('Charlie'), ('Papa'),
('Delta'), ('Quebec'),
('Echo'), ('Romeo'),
('Foxtrot'), ('Sierra'),
('Golf'), ('Tango'),
('Hotel'), ('Uniform'),
('India'), ('Victor'),
('Juliett'), ('Whiskey'),
('Kilo'), ('Xray'),
('Lima'), ('Yankee'),
('Mike'), ('Zulu')
"#;

    #[tokio::test]
    async fn count_total() {
        run_db_test(
            Tbl::TABLE,
            CREATE,
            Some(VALUES),
            |mut pool| async move {
                let mut all = Select::default();
                let mut with_y = Select::default().fields_containing([Tbl::name], "y");

                all.offset = Some(0);
                all.take_n = Some(99);
                all.count_total = Some(true);
                let Paginated { items, total } = Tbl::select_paginated(&all, &mut pool).await.unwrap();
                assert_eq!(items.len(), 26);
                assert_eq!(total, Some(26));

                with_y.offset = Some(0);
                with_y.take_n = Some(99);
                with_y.count_total = Some(true);
                let Paginated { items, total } = Tbl::select_paginated(&with_y, &mut pool).await.unwrap();
                assert_eq!(items.len(), 2);
                assert_eq!(total, Some(2));
            }).await
    }

    #[tokio::test]
    async fn paginated() {
        run_db_test(
            Tbl::TABLE,
            CREATE,
            Some(VALUES),
            |mut pool| async move {
                const CHUNK_SIZE: usize = 7;
                let mut all = Select::default();
                let mut offset = 0;
                all.offset = Some(offset);
                all.take_n = Some(CHUNK_SIZE);
                all.count_total = Some(true);
                let mut item_set = ahash::AHashSet::new();

                loop {
                    let Paginated { items, total } = Tbl::select_paginated(&all, &mut pool).await.unwrap();

                    if matches!(all.count_total, Some(true)) {
                        assert_eq!(total, Some(26));
                        all.count_total = Some(false);
                    } else {
                        assert_eq!(total, None);
                    }

                    let len = items.len();
                    assert!(len <= CHUNK_SIZE, "unexpected len {len}");
                    offset += len;

                    item_set.extend(items);

                    if len < CHUNK_SIZE {
                        break;
                    }

                    all.offset = Some(offset);
                }

                assert_eq!(offset, 26);
                assert_eq!(item_set.len(), 26);
            }).await
    }

}

}
}

mod without_aggr {
    pub(self) use super::*;
    derive_tests!();
}
mod with_aggr {
    pub(self) use super::*;
    derive_tests!(item_aggr_insert);
}

mod versioning_multitask {
    use super::*;
    use crate::db_item::{DbItem, DbItemExt};
    use crate::PgDbOptions;
    use sqlx::PgPool;

    use std::sync::Arc;

    async fn insert_with_lock(
        idx: u32,
        exclusive: bool,
        pool: Arc<PgPool>,
    ) -> Result<()> {
        let mut t = pool.begin().await?;
        if exclusive {
            sqlx::query("lock table version_x in access exclusive mode")
                .execute(&mut t)
                .await?;
        }
        let max = sqlx::query("select count(v) from version_x where id=99")
            .try_map(|x| <(i64,)>::from_row(&x))
            .fetch_one(&mut t)
            .await?
            .0 as i16
            + 1;
        sqlx::query(&format!(
            "insert INTO version_x(id,v,status)
        values(
            99,
            {max},
            {idx}
        );"
        ))
        .execute(&mut t)
        .await
        .map_err(|x| {
            println!("{x}");
            x
        })?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        t.commit().await.map_err(Into::into)
    }

    /// Get a pool. We need this since the default setup gets a transaction.
    async fn get_pool() -> Arc<PgPool> {
        let opt = PgDbOptions::from_env()
            .expect("Не удается получить переменные среды для `PgDbOptions`");
        let pool =
            opt.get_create_pool(true).await.expect("Невозможно создать PgPool");
        Arc::new(pool)
    }

    #[derive(DbItem, DbItemExt, DbVersioned, Debug, Clone, Copy, PartialEq)]
    #[item_table = "moose2"]
    #[db_version_table = "moose_version2"]
    struct Moose {
        #[item_field_pkey]
        uuid: Uuid,
        id: i64,
        rank: i32,
    }

    /// Here we test the principle of multithreading that we use in the program.
    #[tokio::test(flavor = "multi_thread")]
    async fn exclusive_access_test() {
        let pool = get_pool().await;

        sqlx::query("drop table if exists version_x")
            .execute(&*pool)
            .await
            .unwrap();
        sqlx::query(
            "create table if not exists version_x(id integer, v smallint, status smallint, primary key (id, v))"
        ).execute(&*pool).await.unwrap();

        ///////////////////////////////////////////
        // Проверить что падает без эксклюсивности.
        ///////////////////////////////////////////
        let mut handles = Vec::new();
        for x in 5..15 {
            let pool = pool.clone();
            handles.push(tokio::task::spawn(async move {
                insert_with_lock(x, false, pool).await
            }));
        }
        tokio::time::sleep(std::time::Duration::from_millis(5_000)).await;
        let mut errors = 0;
        for h in handles {
            errors += h.await.unwrap().is_err() as u32;
        }
        assert_ne!(errors, 0);
        // Иногда что то вставляется.
        sqlx::query("truncate table version_x").execute(&*pool).await.unwrap();

        ///////////////////////////////////////////
        // Проверить что НЕ падает с ней.
        ///////////////////////////////////////////
        let mut handles = Vec::new();
        for x in 5..15 {
            let pool = pool.clone();
            handles.push(tokio::task::spawn(async move {
                insert_with_lock(x, true, pool).await
            }));
        }
        tokio::time::sleep(std::time::Duration::from_millis(5_000)).await;
        for h in handles {
            assert!(h.await.unwrap().is_ok());
        }

        let x = sqlx::query("select * from version_x order by id,v")
            // status is not stable since lock acquisition time is chaotic.
            .try_map(|x| <(i32, i16, i16)>::from_row(&x).map(|x| (x.0, x.1)))
            .fetch_all(&*pool)
            .await
            .unwrap();
        assert_eq!(
            x,
            vec![
                (99, 1),
                (99, 2),
                (99, 3),
                (99, 4),
                (99, 5),
                (99, 6),
                (99, 7),
                (99, 8),
                (99, 9),
                (99, 10),
            ]
        );
    }
}

mod update_with_autogen {
    pub(self) use super::*;

    #[derive(Clone, Debug, Default, DbItem)]
    struct Header1 {
        uuid: Uuid,
        #[item_field_pkey]
        #[item_field_autogen]
        id: i32,
        flag: bool,
    }

    #[derive(Clone, Debug, Default, DbItem)]
    struct Header2 {
        uuid: Uuid,
        #[item_field_pkey]
        #[item_field_autogen_always]
        id: i32,
        flag: bool,
    }

    const CREATE1: &str =
        "(uuid UUID, id INTEGER GENERATED BY DEFAULT AS IDENTITY, flag BOOL)";
    const CREATE2: &str =
        "(uuid UUID, id INTEGER GENERATED ALWAYS AS IDENTITY, flag BOOL)";
    const INSERT: &str = "(flag) values (true), (true), (false)";

    macro_rules! to_update {
        ($table:ident) => {
            [
                $table {
                    id: 1,
                    flag: false,
                    ..Default::default()
                },
                $table {
                    id: 2,
                    flag: false,
                    ..Default::default()
                },
                $table {
                    id: 3,
                    flag: true,
                    ..Default::default()
                },
            ]
        };
    }

    #[tokio::test]
    async fn update_with_autogen() {
        run_db_test(Header1::TABLE, CREATE1, Some(INSERT), |mut pool| async move {
            let to_update = to_update!(Header1);

            Header1::update_vec(&to_update, Some(&[Header1::flag]), &mut pool)
                .await
                .expect("ok");
        })
        .await;
    }

    #[tokio::test]
    async fn update_with_autogen_set_autogen() {
        run_db_test(Header1::TABLE, CREATE1, Some(INSERT), |mut pool| async move {
            let to_update = to_update!(Header1);

            Header1::update_vec(
                &to_update,
                Some(&[Header1::flag, Header1::id]),
                &mut pool,
            )
            .await
            .expect("ok");
        })
        .await;
    }

    #[tokio::test]
    async fn update_with_autogen_always() {
        run_db_test(Header2::TABLE, CREATE2, Some(INSERT), |mut pool| async move {
            let to_update = to_update!(Header2);

            Header2::update_vec(&to_update, Some(&[Header2::flag]), &mut pool)
                .await
                .expect("ok");
        })
        .await;
    }
}

mod array_filter {
    use crate::db_item::int_array::AsezArray;

    use super::*;

    #[derive(Clone, Debug, Default, DbItem, PartialEq)]
    struct Item {
        #[item_field_pkey]
        id: i32,
        a16: AsezArray<i16>,
        a32: AsezArray<i32>,
        a64: AsezArray<i64>,
    }

    const CREATE: &str =
        "(id INTEGER, a16 SMALLINT[], a32 INTEGER[], a64 BIGINT[])";
    const INSERT: &str = "(id, a16, a32, a64) VALUES
            (1, array[1, 2, 3], array[1000, 2000, 3000], array[1000000, 2000000, 3000000]),
            (2, array[4, 5, 6], array[4000, 5000, 6000], array[4000000, 5000000, 6000000]),
            (3, array[7, 8, 9], array[7000, 8000, 9000], array[7000000, 8000000, 9000000])
            ";

    #[tokio::test]
    async fn select_i64_array() {
        run_db_test(Item::TABLE, CREATE, Some(INSERT), |mut pool| async move {
            let items = Item::select(
                &Select::full::<Item>()
                    .eq(Item::a64, AsezArray(vec![1000000_i64, 2000000, 3000000])),
                &mut pool,
            )
            .await
            .expect("ok");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].id, 1);
        })
        .await
    }

    #[tokio::test]
    #[ignore = "in_any does not work with arrays"]
    async fn select_i64_array_in() {
        run_db_test(Item::TABLE, CREATE, Some(INSERT), |mut pool| async move {
            let value = AsezArray(vec![1000000_i64, 2000000, 3000000]);
            let items = Item::select(
                &Select::full::<Item>().in_any(Item::a64, [value]),
                &mut pool,
            )
            .await
            .expect("ok");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].id, 1);
        })
        .await
    }

    #[tokio::test]
    async fn select_i32_array() {
        run_db_test(Item::TABLE, CREATE, Some(INSERT), |mut pool| async move {
            let items = Item::select(
                &Select::full::<Item>()
                    .eq(Item::a32, AsezArray(vec![4000_i32, 5000, 6000])),
                &mut pool,
            )
            .await
            .expect("ok");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].id, 2);
        })
        .await
    }

    #[tokio::test]
    async fn select_i16_array() {
        run_db_test(Item::TABLE, CREATE, Some(INSERT), |mut pool| async move {
            let items = Item::select(
                &Select::full::<Item>().eq(Item::a16, AsezArray(vec![7_i16, 8, 9])),
                &mut pool,
            )
            .await
            .expect("ok");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].id, 3);
        })
        .await
    }

    #[tokio::test]
    #[ignore = "in_any does not work with arrays"]
    async fn select_i16_array_in() {
        run_db_test(Item::TABLE, CREATE, Some(INSERT), |mut pool| async move {
            let value1 = AsezArray(vec![7_i16, 8, 9]);
            let value2 = AsezArray(vec![9_i16, 8, 7]);
            let items = Item::select(
                &Select::full::<Item>().in_any(Item::a16, [value1, value2]),
                &mut pool,
            )
            .await
            .expect("ok");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].id, 3);
        })
        .await
    }
}

mod update_by_filter {
    use super::*;
    use crate::db_item::DbUpdateByFilter;

    #[derive(Clone, Debug, Default, DbItem, PartialEq)]
    struct Item {
        #[item_field_pkey]
        id: i32,
        name: String,
        year: i16,
        price: i64,
        code: i16,
    }
    impl DbUpdateByFilter for Item {}
    const INSERT: &str = "(id, name, year, price, code) VALUES
        (1,'Bob',2065,1000,1),
        (2,'Alice',2065,1000,2),
        (3,'Bob',2065,2000,3),
        (4,'Alice',2065,2000,4),
        (5,'Bob',2064,1000,5),
        (6,'Alice',2064,1000,6),
        (7,'Bob',2064,1000,7),
        (8,'Alice',2064,1000,8)
    ";
    fn expected() -> Vec<Item> {
        vec![
            Item {
                id: 1,
                name: "Bob".to_string(),
                year: 2065,
                price: 9000,
                code: -1,
            },
            Item {
                id: 3,
                name: "Bob".to_string(),
                year: 2065,
                price: 9000,
                code: -1,
            },
            Item {
                id: 5,
                name: "Bob".to_string(),
                year: 2064,
                price: 1000,
                code: 5,
            },
            Item {
                id: 7,
                name: "Bob".to_string(),
                year: 2064,
                price: 1000,
                code: 7,
            },
        ]
    }

    #[tokio::test]
    async fn test_update_by_filter_ret() {
        run_db_test(
            Item::TABLE,
            "(id INTEGER, name TEXT, year SMALLINT, price BIGINT, code SMALLINT)",
            Some(INSERT),
            |mut pool| async move {
                let item = Item {
                    price: 9000,
                    code: -1,
                    ..Item::default()
                };
                let filters = FilterTree::and_from_list(vec![
                    Filter::eq(Item::year, 2065),
                    Filter::eq(Item::name, "Bob"),
                ]);
                let mut ret = item
                    .update_by_filter_returning(
                        &[Item::price, Item::code],
                        &filters,
                        Some(Item::FIELDS),
                        &mut pool,
                    )
                    .await
                    .unwrap();
                ret.sort_by(|a, b| a.id.cmp(&b.id));

                assert_eq!(ret.len(), 2);
                assert_eq!(
                    ret,
                    vec![
                        Item {
                            id: 1,
                            name: "Bob".to_string(),
                            year: 2065,
                            price: 9000,
                            code: -1,
                        },
                        Item {
                            id: 3,
                            name: "Bob".to_string(),
                            year: 2065,
                            price: 9000,
                            code: -1,
                        },
                    ]
                );
                // Extra check
                let s = Select::full::<Item>().eq(Item::name, "Bob");
                let mut check_items = Item::select(&s, &mut pool).await.unwrap();
                check_items.sort_by(|a, b| a.id.cmp(&b.id));
                assert_eq!(check_items, expected());
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_update_by_filter() {
        run_db_test(
            Item::TABLE,
            "(id INTEGER, name TEXT, year SMALLINT, price BIGINT, code SMALLINT)",
            Some(INSERT),
            |mut pool| async move {
                let item = Item {
                    price: 9000,
                    code: -1,
                    ..Item::default()
                };
                let filters = FilterTree::and_from_list(vec![
                    Filter::eq(Item::year, 2065),
                    Filter::eq(Item::name, "Bob"),
                ]);
                let ret = item
                    .update_by_filter(
                        &[Item::price, Item::code],
                        &filters,
                        &mut pool,
                    )
                    .await
                    .unwrap();

                assert_eq!(ret, 2);
                // Extra check
                let s = Select::full::<Item>().eq(Item::name, "Bob");
                let mut check_items = Item::select(&s, &mut pool).await.unwrap();
                check_items.sort_by(|a, b| a.id.cmp(&b.id));
                assert_eq!(check_items, expected());
            },
        )
        .await;
    }
}
