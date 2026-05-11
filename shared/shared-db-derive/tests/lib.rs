use asez2_shared_db::db_item::DbFieldMask;
use asez2_shared_db::db_item::DbItemExt as DbItemExtStuffing;
use asez2_shared_db::db_item::DbVersioned as DbVersionedExtra;
use asez2_shared_db::db_item::FieldTolerance;
use asez2_shared_db::DbAdaptor as DbAdaptorStuffing;
use asez2_shared_db::DbItem as DbItemStuffing;
use serde::{Deserialize, Serialize};
use shared_db_derive::{
    DbAdaptor, DbEnum, DbItem, DbItemExt, DbUpsert, DbVersioned,
};

fn falsify(b: bool) -> bool {
    !b
}

macro_rules! derive_tests {
    ($($aggr_arrays:ident)?) => {
mod giant_hedgehog {
    use super::*;
    use std::fmt::Display;

    fn falsify(b: bool) -> bool {
        !b
    }

    fn local_parse(n: String) -> Result<i64, String> {
        let r = n.parse().map_err(|_| "Could not parse error".to_string())?;
        Ok(r)
    }

    fn local_to_string<T: Display>(s: T) -> String {
        s.to_string()
    }

    #[derive(DbAdaptor, DbItem, Debug, Clone)]
    #[adaptor_rename = "GiantHedgehog"]
    #[adaptor_derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
    #[item_table = "hedgehogs"]
    $(#[$aggr_arrays])?
    struct MyStruct {
        #[item_field_pkey]
        id: i64,
        #[adaptor_rename = "spineless"]
        #[adaptor_from = "super::falsify"]
        #[adaptor_into = "falsify"]
        spines: bool,
        #[adaptor_rename = "species"]
        name: String,
        #[adaptor_type = "String"]
        #[adaptor_into = "local_to_string"]
        #[adaptor_try_from = "local_parse"]
        #[adaptor_rename = "mass_g"]
        weight: i64,
    }

    #[test]
    fn test_derive_simple_struct() {
        use asez2_shared_db::DbAdaptor;

        let a = MyStruct {
            id: 99,
            spines: false,
            name: String::from("Hedgy"),
            weight: 54,
        };
        let hedgy: GiantHedgehog = GiantHedgehog::from_item::<&str>(a.clone(), None);
        let limited_hedgy: GiantHedgehog = GiantHedgehog::from_item(
            a,
            Some(&["id", "name"])
        );
        let new_thing = GiantHedgehog {
            id: Some(99),
            spineless: Some(true),
            species: Some(String::from("Hedgy")),
            mass_g: Some(String::from("54")),
        };
        let limited_thing = GiantHedgehog {
            id: Some(99),
            spineless: None,
            species: Some(String::from("Hedgy")),
            mass_g: None,
        };

        assert_eq!(hedgy, new_thing);
        assert_eq!(limited_hedgy, limited_thing);
    }
}

mod default_hedgehog {
    use super::*;
    use asez2_shared_db::db_item::DbAdaptor as DbAdaptor2;

    #[derive(DbAdaptor, DbItem, Debug, Clone)]
    #[adaptor_rename = "GiantHedgehog"]
    #[adaptor_derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
    #[item_table = "hedgehogs"]
    // sqlx default is needed to test branches in something.
    $(#[$aggr_arrays])?
    struct MyStruct {
        #[item_field_pkey]
        id: i64,
        spines: bool,
        name: String,
        weight: i64,
    }

    #[test]
    fn test_derive_simple_struct() {
        use asez2_shared_db::DbAdaptor;

        let a = MyStruct {
            id: 99,
            spines: false,
            name: String::from("Hedgy"),
            weight: 54,
        };
        let hedgy: GiantHedgehog = GiantHedgehog::from_item::<&str>(a.clone(), None);
        let limited_hedgy: GiantHedgehog = GiantHedgehog::from_item(
            a,
            Some(&["id", "name"])
        );

        let new_thing = GiantHedgehog {
            id: Some(99),
            spines: Some(false),
            name: Some(String::from("Hedgy")),
            weight: Some(54),
        };
        let limited_thing = GiantHedgehog {
            id: Some(99),
            spines: None,
            name: Some(String::from("Hedgy")),
            weight: None,
        };

        assert_eq!(hedgy, new_thing);
        assert_eq!(limited_hedgy, limited_thing);
    }

    #[test]
    fn test_create_default_bind_mask1() {
        let hedgehogs = vec![
            GiantHedgehog {
                id: Some(99),
                spines: Some(false),
                name: Some(String::from("Hedgy")),
                weight: None,
            },
            GiantHedgehog {
                id: Some(99),
                spines: None,
                name: Some(String::from("Hedgy")),
                weight: None,
            },
            GiantHedgehog {
                id: Some(99),
                spines: None,
                name: Some(String::from("Hedgy")),
                weight: None,
            },
            GiantHedgehog {
                id: Some(99),
                spines: None,
                name: Some(String::from("Hedgy")),
                weight: None,
            },
            GiantHedgehog {
                id: Some(99),
                spines: None,
                name: Some(String::from("Hedgy")),
                weight: Some(99),
            },
        ];

        let mask = GiantHedgehog::create_default_bind_mask(&hedgehogs);
        assert_eq!(mask, [true, true, true, true]);

        let mask = GiantHedgehog::create_strict_bind_mask(&hedgehogs);
        assert_eq!(
            &mask.unwrap_err().to_string(),
            "`spines` is not present in all items."
        );
    }

    #[test]
    fn test_create_default_bind_mask2() {
        let hedgehogs = vec![
            GiantHedgehog {
                id: None,
                spines: None,
                name: None,
                weight: None,
            },
            GiantHedgehog {
                id: None,
                spines: None,
                name: None,
                weight: None,
            },
            GiantHedgehog {
                id: None,
                spines: None,
                name: None,
                weight: None,
            },
            GiantHedgehog {
                id: None,
                spines: None,
                name: None,
                weight: None,
            }
            ,GiantHedgehog {
                id: None,
                spines: None,
                name: None,
                weight: None,
            },
        ];

        let mask = GiantHedgehog::create_default_bind_mask(&hedgehogs);
        assert_eq!(mask, DbFieldMask::none());

        let mask = GiantHedgehog::create_strict_bind_mask(&hedgehogs).unwrap();
        assert_eq!(mask, DbFieldMask::none());
    }

    #[test]
    fn test_create_default_bind_mask3() {
        let hedgehogs = vec![
            GiantHedgehog {
                id: Some(99),
                spines: None,
                name: Some(String::from("Hedgy")),
                weight: None,
            },
            GiantHedgehog {
                id: Some(99),
                spines: None,
                name: Some(String::from("Hedgy")),
                weight: None,
            },
            GiantHedgehog {
                id: Some(99),
                spines: None,
                name: Some(String::from("Hedgy")),
                weight: None,
            },
            GiantHedgehog {
                id: Some(99),
                spines: None,
                name: Some(String::from("Hedgy")),
                weight: None,
            },
            GiantHedgehog {
                id: Some(99),
                spines: None,
                name: Some(String::from("Hedgy")),
                weight: None,
            },
        ];

        let mask = GiantHedgehog::create_default_bind_mask(&hedgehogs);
        assert_eq!(mask, [true, false, true, false]);

        let mask = GiantHedgehog::create_strict_bind_mask(&hedgehogs).unwrap();
        assert_eq!(mask, [true, false, true, false]);
    }
}

/// The tests here are not designed to test the complete functionality of the
/// `DbItem` derive macro. They are supposed to test basic functionality
/// of the macro and tags.
#[allow(dead_code)]
mod db_item_basics {
    use super::*;

    #[derive(DbItem, Debug, Clone)]
    #[item_table = "table1"]
    $(#[$aggr_arrays])?
    struct StandardTable {
        #[item_field_pkey]
        #[item_field_require_from_row]
        my_id: i32,
        my_value: String,
    }

    #[derive(DbItem, Debug, Clone)]
    #[item_table = "table2"]
    $(#[$aggr_arrays])?
    struct StandardTable2 {
        #[item_field_pkey]
        my_id: i32,
        my_value: String,
    }

    fn default_other_value() -> Option<String> {
        Some(String::from("I am active."))
    }

    #[derive(DbItem, Debug, Clone)]
    #[item_table = "table3"]
    $(#[$aggr_arrays])?
    struct StandardTable3 {
        #[item_field_pkey]
        my_id: i32,
        my_value: String,
        #[item_field_activate_with = "default_other_value()"]
        my_other_value: Option<String>,
    }

    #[derive(DbItem, Debug, Clone)]
    #[item_table = "table4"]
    $(#[$aggr_arrays])?
    struct StandardTable4 {
        #[item_field_pkey]
        #[item_field_autogen]
        #[item_field_require_from_row]
        my_id: i32,
        #[item_field_require_from_row]
        my_value: String,
        my_other_value: Option<String>,
    }

    // Not really random.
    fn default_planet() -> String {
        "earth".to_string()
    }

    #[derive(DbItem, Debug, Clone)]
    #[item_table = "interplanetary"]
    #[item_activate_all_with = "default_planet()"]
    $(#[$aggr_arrays])?
    struct StandardTable5 {
        #[item_field_pkey]
        planetary_origin: String,
        my_value: String,
    }

    #[test]
    fn test_standard_tables() {
        use asez2_shared_db::DbItem;

        assert_eq!(StandardTable::FIELDS, &["my_id", "my_value"]);
        assert_eq!(StandardTable2::FIELDS, &["my_id", "my_value"]);
        assert_eq!(
            StandardTable3::FIELDS,
            &["my_id", "my_value", "my_other_value"]
        );
        assert_eq!(
            StandardTable4::FIELDS,
            &["my_id", "my_value", "my_other_value"]
        );
        assert_eq!(StandardTable5::FIELDS, &["planetary_origin", "my_value"]);
    }

    #[test]
    fn test_stand_pkeys() {
        use asez2_shared_db::DbItem;

        assert_eq!(StandardTable::PRIMARY_KEYS, &["my_id"]);
        assert_eq!(StandardTable2::PRIMARY_KEYS, &["my_id"]);
        assert_eq!(StandardTable3::PRIMARY_KEYS, &["my_id"]);
        assert_eq!(StandardTable4::PRIMARY_KEYS, &["my_id"]);
        assert_eq!(StandardTable5::PRIMARY_KEYS, &["planetary_origin"]);
    }

    #[test]
    fn test_stand_table() {
        use asez2_shared_db::DbItem;

        assert_eq!(StandardTable::TABLE, "table1");
        assert_eq!(StandardTable2::TABLE, "table2");
        assert_eq!(StandardTable3::TABLE, "table3");
        assert_eq!(StandardTable4::TABLE, "table4");
        assert_eq!(StandardTable5::TABLE, "interplanetary");
    }
}

/// As with `db_item_basics`, these tests are more compile time tests
/// that are designed to test basic functionality of the derive macro.
/// At most they are small roundtrip tests.
mod db_adaptor_basics {
    use super::*;
    use asez2_shared_db::DbAdaptor as DbAdaptor2;

    #[test]
    fn test_standard_table() {
        #[derive(DbItem, DbAdaptor, Debug, Clone, PartialEq)]
        #[adaptor_derive(Debug, Deserialize, Serialize, PartialEq, Default)]
        #[item_table = "standard_table"]
        struct StandardTable {
            #[item_field_pkey]
            my_id: i32,
            my_value: String,
        }

        let table = StandardTable {
            my_id: 5,
            my_value: "Home".to_owned(),
        };
        let rep = StandardTableRep::from_item::<&str>(table.clone(), None);
        let exp = StandardTableRep {
            my_id: Some(5),
            my_value: Some("Home".to_owned()),
        };

        assert_eq!(rep, exp);

        let table_ret = rep.into_item().unwrap();
        assert_eq!(table_ret, table)
    }

    #[test]
    fn test_standard_table2() {
        #[derive(DbItem, DbAdaptor, Debug, Clone, PartialEq)]
        #[adaptor_derive(Debug, Deserialize, Serialize, PartialEq, Default)]
        #[adaptor_rename = "IrcPost"]
        #[item_table = "standard_table"]
        struct StandardTable2 {
            #[item_field_pkey]
            my_id: i32,
            #[adaptor_rename = "message_content"]
            my_value: String,
        }

        let table = StandardTable2 {
            my_id: 5,
            my_value: "Home".to_owned(),
        };
        let rep = IrcPost::from_item::<&str>(table.clone(), None);
        let exp = IrcPost {
            my_id: Some(5),
            message_content: Some("Home".to_owned()),
        };

        assert_eq!(rep, exp);

        let table_ret = rep.into_item().unwrap();
        assert_eq!(table_ret, table)
    }

    #[test]
    fn test_standard_table3() {
        fn stupid_manual_as_i32(x: i64) -> i32 {
            x as i32
        }

        #[derive(DbItem, DbAdaptor, Debug, Clone, PartialEq)]
        #[adaptor_derive(Debug, Deserialize, Serialize, PartialEq, Default)]
        #[item_table = "standard_table"]
        struct StandardTable3 {
            #[item_field_pkey]
            #[adaptor_type = "i64"]
            #[adaptor_from = "stupid_manual_as_i32"]
            my_id: i32,
            my_value: String,
            #[adaptor_type = "String"]
            #[adaptor_from = "Some"]
            #[adaptor_into = "Option::unwrap_or_default"]
            my_other_value: Option<String>,
        }

        let mut table = StandardTable3 {
            my_id: 5,
            my_value: "Home".to_owned(),
            my_other_value: Some("OtherHome".to_owned()),
        };
        let rep = StandardTable3Rep::from_item::<&str>(table.clone(), None);
        let exp = StandardTable3Rep {
            my_id: Some(5),
            my_value: Some("Home".to_owned()),
            my_other_value: Some("OtherHome".to_owned()),
        };

        assert_eq!(rep, exp);

        let table_ret = rep.into_item().unwrap();
        assert_eq!(table_ret, table);

        table.my_other_value = None;
        let rep = StandardTable3Rep::from_item::<&str>(table, None);
        let exp = StandardTable3Rep {
            my_id: Some(5),
            my_value: Some("Home".to_owned()),
            my_other_value: Some("".to_owned()),
        };

        assert_eq!(rep, exp);
    }

    #[test]
    fn test_standard_table4() {
        #[derive(DbItem, DbAdaptor, Debug, Clone, PartialEq)]
        #[adaptor_derive(Debug, Deserialize, Serialize, PartialEq, Default)]
        #[adaptor_attribute_for_all(#[serde(default)])]
        #[item_table = "standard_table"]
        struct StandardTable5 {
            #[item_field_pkey]
            planetary_origin: String,
            my_value: String,
        }

        let table = StandardTable5 {
            planetary_origin: "5".to_owned(),
            my_value: "Home".to_owned(),
        };
        let rep = StandardTable5Rep::from_item::<&str>(table.clone(), None);
        let exp = StandardTable5Rep {
            planetary_origin: Some("5".to_owned()),
            my_value: Some("Home".to_owned()),
        };

        assert_eq!(rep, exp);

        let table_ret = rep.into_item().unwrap();
        assert_eq!(table_ret, table);

        let output: StandardTable5Rep = serde_json::from_str("{}").unwrap();
        assert_eq!(
            output,
            StandardTable5Rep {
                planetary_origin: None,
                my_value: None,
            },
            "3rd assert failed"
        );
    }

    #[test]
    fn test_standard_table5() {
        fn special_planet() -> Option<String> {
            Some("EARTH!!".to_owned())
        }
        #[derive(DbItem, DbAdaptor, Debug, Clone, PartialEq)]
        #[adaptor_derive(Debug, Deserialize, Serialize, PartialEq, Default)]
        #[adaptor_attribute_for_all(#[serde(default)])]
        #[item_table = "standard_table"]
        struct StandardTable5 {
            #[item_field_pkey]
            #[adaptor_attributes(#[serde(default = "special_planet")])]
            planetary_origin: String,
            #[adaptor_rename = "message_content"]
            my_value: String,
        }

        let table = StandardTable5 {
            planetary_origin: "5".to_owned(),
            my_value: "Home".to_owned(),
        };
        let rep = StandardTable5Rep::from_item::<&str>(table.clone(), None);
        let exp = StandardTable5Rep {
            planetary_origin: Some("5".to_owned()),
            message_content: Some("Home".to_owned()),
        };

        assert_eq!(rep, exp, "1st assert failed.");

        let table_ret = rep.into_item().unwrap();
        assert_eq!(table_ret, table, "2nd assert failed.");

        let output: StandardTable5Rep = serde_json::from_str("{}").unwrap();
        assert_eq!(
            output,
            StandardTable5Rep {
                planetary_origin: Some(String::from("EARTH!!")),
                message_content: None,
            },
            "3rd assert failed."
        );
    }

    #[test]
    fn serde_tests() {
        #[derive(DbItem, DbAdaptor, Debug, Clone, PartialEq)]
        #[adaptor_derive(Debug, Deserialize, Serialize, PartialEq, Default)]
        #[item_table = "standard_table"]
        struct StTable {
            #[item_field_pkey]
            key: i16,
            my_value: Option<i64>,
        }

        let a = StTableRep {
            key: Some(1),
            my_value: Some(Some(59)),
        };
        let b = StTableRep {
            key: Some(2),
            my_value: Some(None),
        };
        let c = StTableRep {
            key: Some(3),
            my_value: None,
        };
        let d = StTableRep {
            key: None,
            my_value: None,
        };

        let a_string = serde_json::to_string(&a).unwrap();
        let b_string = serde_json::to_string(&b).unwrap();
        let c_string = serde_json::to_string(&c).unwrap();
        let d_string = serde_json::to_string(&d).unwrap();

        assert_eq!(r#"{"key":1,"my_value":59}"#, &a_string);
        assert_eq!(r#"{"key":2,"my_value":null}"#, &b_string);
        assert_eq!(r#"{"key":3}"#, &c_string);
        assert_eq!(r#"{}"#, &d_string);

        assert_eq!(a, serde_json::from_str(&a_string).unwrap());
        assert_eq!(b, serde_json::from_str(&b_string).unwrap());
        assert_eq!(c, serde_json::from_str(&c_string).unwrap());
        assert_eq!(d, serde_json::from_str(&d_string).unwrap());

    }

    #[test]
    fn duplicate_field_tests() {
        #[derive(DbItem, DbAdaptor, Debug, Clone, PartialEq)]
        #[adaptor_derive(Debug, Deserialize, Serialize, PartialEq, Default)]
        #[item_table = "standard_table"]
        struct DupTable {
            #[item_field_pkey]
            key: i16,
            #[adaptor_field_duplicate = "my_value_short"]
            my_value: String,
        }

        let rep_a = DupTableRep {
            key: Some(1),
            my_value: Some("My favourite field".to_string()),
            my_value_short: Some("My favourite field".to_string()),
        };
        let exp_a = DupTable {
            key: 1,
            my_value: "My favourite field".to_string(),
        };
        let item_b = DupTable {
            key: 99,
            my_value: "My other favourite field".to_string(),
        };
        let exp_rep_b = DupTableRep {
            key: Some(99),
            my_value: Some("My other favourite field".to_string()),
            my_value_short: Some("My other favourite field".to_string()),
        };

        assert_eq!(rep_a.into_item().unwrap(), exp_a);
        assert_eq!(DupTableRep::from_item::<&str>(item_b, None), exp_rep_b);

    }
}

mod db_adaptor_merge_tests {
    use super::*;
    use uuid::Uuid;

    #[derive(DbItem, DbAdaptor, Debug, Clone, PartialEq)]
    #[adaptor_derive(Debug,Default,Clone,PartialEq, serde::Serialize,serde::Deserialize)]
    #[item_table = "hedgehogs"]
    $(#[$aggr_arrays])?
    struct MyItem {
        #[item_field_pkey]
        uuid: Uuid,
        spines: bool,
        name: String,
        weight: std::option::Option<i64>,
    }
    #[test]
    fn test_merge() {
        let uuid = uuid::Uuid::new_v4();
        let item = MyItem {
            uuid,
            spines: true,
            name: "My Name".to_string(),
            weight: Some(100),
        };

        let dto = MyItemRep {
            weight: Some(None),
            spines: Some(false),
            ..Default::default()
        };

        let new_item = dto.into_item_merged(
            item,
        ).expect("success");
        assert_eq!(new_item, MyItem {
            uuid,
            spines: false,
            name: "My Name".to_string(),
            weight: None,
        })
    }

    #[test]
    fn test_zero() {
        let uuid = uuid::Uuid::new_v4();
        let item = MyItemRep {
            uuid: Some(uuid),
            spines: Some(true),
            name: None,
            weight: Some(Some(100)),
        };
        let exp = MyItemRep {
            uuid: Some(uuid),
            spines: Some(false),
            name: Some(String::default()),
            weight: Some(Some(100)),
        };
        // zero names and spines.
        let to_drop = &["spines", "name"];
        let after = item.zero_fields(&DbFieldMask::with_fields(to_drop));
        assert_eq!(after, exp);
    }

    #[test]
    fn test_drop_fields() {
        let uuid = uuid::Uuid::new_v4();
        let item = MyItemRep {
            uuid: Some(uuid),
            spines: Some(true),
            name: Some("asdda".to_owned()),
            weight: Some(Some(100)),
        };
        let exp = MyItemRep {
            uuid: Some(uuid),
            spines: None,
            name: Some("asdda".to_owned()),
            weight: None,
        };
        // drop  spines and weight.
        let to_drop = &["spines", "weight"];
        let after = item.unset_fields(&DbFieldMask::with_fields(to_drop));
        assert_eq!(after, exp);
    }
}

mod db_item_ext_tests {
    use super::*;
    use asez2_shared_db::db_item::Field;
    use uuid::Uuid;

    #[derive(DbItem, DbItemExt, Debug, Clone)]
    #[item_table = "hedgehogs"]
    $(#[$aggr_arrays])?
    struct MyStruct {
        #[item_field_pkey]
        uuid: Uuid,
        spines: bool,
        name: String,
        weight: std::option::Option<i64>,
    }

    fn new_struct() -> MyStruct {
        MyStruct {
            uuid: uuid::Uuid::new_v4(),
            spines: false,
            name: "Not a Hog".to_string(),
            // antigrav hedgehogs.
            weight: Some(-9999),
        }
    }


    #[test]
    fn test_all_fields() {
        let hog = new_struct();

        let all_fields = hog.fields_with_values();

        assert_eq!(all_fields.len(), 4);
        let expected_fields = vec![
            Field::new("uuid", Some(hog.uuid)),
            Field::new("spines", Some(false)),
            Field::new("name", Some("Not a Hog".to_string())),
            Field::new("weight", Some(-9999)),
        ];
        assert_eq!(all_fields, expected_fields);
    }

    #[test]
    fn test_uuid() {
        let hog = new_struct();
        let uuid_val = hog.record_uuid();
        assert_eq!(uuid_val, hog.uuid);
    }

    #[test]
    fn test_okeys_with_value() {
        let hog = new_struct();
        let fields = hog.pkeys_with_values();

        assert_eq!(fields.len(), 1);
        let expected_fields = vec![Field::new("uuid", Some(hog.uuid))];
        assert_eq!(fields, expected_fields);
    }

    #[test]
    fn compare_two_hogs1() {
        let hog = new_struct();
        let hog2 = MyStruct {
            uuid: hog.uuid,
            spines: false,
            name: "Not a Hog".to_string(),
            weight: Some(-9999),
        };
        let changed = hog.differing_fields(&hog2);
        assert!(changed.is_empty());
    }

    #[test]
    fn compare_two_hogs2() {
        let hog = new_struct();
        let hog2 = MyStruct {
            uuid: hog.uuid,
            spines: true,
            name: "I am hog!".to_string(),
            weight: Some(-9999),
        };
        let changed = hog.differing_fields(&hog2);

        assert_eq!(changed.len(), 2);
        let expected_fields = vec![
            Field::new("spines", Some(true)),
            Field::new("name", Some("I am hog!".to_string())),
        ];
        assert_eq!(changed, expected_fields);
    }

    #[test]
    fn compare_two_hogs3() {
        let hog = new_struct();
        let hog2 = MyStruct {
            uuid: uuid::Uuid::new_v4(),
            spines: false,
            name: "Not a Hog".to_string(),
            weight: Some(100),
        };
        let changed = hog.differing_fields(&hog2);

        assert_eq!(changed.len(), 2);
        let expected_fields = vec![
            Field::new("uuid", Some(hog2.uuid)),
            Field::new("weight", Some(100)),
        ];
        assert_eq!(changed, expected_fields);
    }
}

// NB: We do not test this here, we simply check that it compiles.
mod skip_field_tolerance_tests {
    use super::*;

    #[derive(DbItem, DbAdaptor, Debug, Clone)]
    #[adaptor_derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
    #[item_table = "hedgehogs"]
    #[item_skip_field_tolerance]
    $(#[$aggr_arrays])?
    struct MyStruct {
        #[item_field_pkey]
        uuid: i64,
        #[adaptor_field_duplicate = "spikey_wikeys"]
        spines: bool,
        #[adaptor_field_duplicate = "measure_my_needles"]
        needle_len: String,
        weight: std::option::Option<i64>,
    }

    impl FieldTolerance for MyStruct {
        const TOLERATED: &'static[(&'static str, &'static str)] = &[
            ("spikey_wikeys","spines"),
            ("measure_my_needles", "needle_len")
        ];
    }

    #[test]
    fn test_duplicate() {
        let hedge1 = MyStruct {
            uuid: 54,
            spines: true,
            needle_len: "long, very long..".to_string(),
            weight: Some(3),
        };

        let exp_full = MyStructRep {
            uuid: Some(54),
            spines: Some(true),
            spikey_wikeys: Some(true),
            needle_len: Some("long, very long..".to_string()),
            measure_my_needles: Some("long, very long..".to_string()),
            weight: Some(Some(3)),
        };
        let mut exp_old_fields_only = exp_full.clone();
        exp_old_fields_only.spikey_wikeys = None;
        exp_old_fields_only.measure_my_needles = None;
        let mut new_fields_only = exp_full.clone();
        new_fields_only.spines = None;
        new_fields_only.needle_len = None;
        new_fields_only.weight = None;

        let res_full = MyStructRep::from_item::<&str>(hedge1.clone(), None);

        let res_old = MyStructRep::from_item(
            hedge1.clone(),
            Some(MyStruct::FIELDS)
        );

        let res_new = MyStructRep::from_item(
            hedge1.clone(),
            Some(&["spikey_wikeys", "measure_my_needles"])
        );

        assert_eq!(res_full, exp_full, "full failed");
        assert_eq!(res_old, exp_old_fields_only, "old fields only failed");
        assert_eq!(res_new, new_fields_only, "new fields only failed");
    }
}

mod upsert {
    use super::*;

    #[derive(DbItem, DbUpsert, DbAdaptor, Debug, Clone)]
    #[adaptor_derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
    #[item_table = "hedgehogs"]
    $(#[$aggr_arrays])?
    struct MyUpsertStruct {
        #[item_field_pkey]
        uuid: i64,
        #[adaptor_field_duplicate = "spikey_wikeys"]
        spines: bool,
        #[adaptor_field_duplicate = "measure_my_needles"]
        needle_len: String,
        weight: std::option::Option<i64>,
    }

}
    }
}

mod versioned {
    use super::*;

    #[test]
    fn test_hedge_versions() {
        #[derive(DbVersioned, DbItem, Debug, Clone, PartialEq)]
        #[db_version_table = "hedge_version"]
        #[item_table = "hedge"]
        struct Hedge {
            #[item_field_pkey]
            id: i32,
            text: String,
        }

        let h = Hedge {
            id: 32,
            text: "Row".to_string(),
        };

        let exp_v = HedgeVersion {
            pricing_version: -999,
            id: 32,
            text: "Row".to_string(),
        };

        assert_eq!(exp_v, h.to_versioned(-999));
    }

    #[test]
    fn test_hedge2_versions() {
        #[derive(DbVersioned, DbItem, Debug, Clone, PartialEq)]
        #[item_table = "hedge"]
        #[db_version_table = "hedge_version"]
        #[versioned = "AncientHedgesOfRome"]
        struct Hedge {
            #[item_field_pkey]
            id: i32,
            text: String,
        }

        let h = Hedge {
            id: 32,
            text: "Row".to_string(),
        };

        let exp_v = AncientHedgesOfRome {
            pricing_version: 100,
            id: 32,
            text: "Row".to_string(),
        };

        assert_eq!(exp_v, h.to_versioned(100));
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

#[test]
fn db_enumeration_test() {
    use asez2_shared_db::Value;

    #[derive(DbEnum, PartialEq, Eq, Debug)]
    #[repr(i16)]
    enum SomeStatus {
        #[db_default]
        Undefined = 0,
        Success = 1,
        Failure = 2,
    }

    assert_eq!(SomeStatus::default(), SomeStatus::Undefined);

    assert_eq!(SomeStatus::from(0), SomeStatus::Undefined);
    assert_eq!(SomeStatus::from(1), SomeStatus::Success);
    assert_eq!(SomeStatus::from(2), SomeStatus::Failure);

    assert_eq!(i16::from(SomeStatus::Undefined), 0);
    assert_eq!(i16::from(SomeStatus::Success), 1);
    assert_eq!(i16::from(SomeStatus::Failure), 2);

    assert_eq!(Value::from(SomeStatus::Undefined), Value::Int(0));
    assert_eq!(Value::from(SomeStatus::Success), Value::Int(1));
    assert_eq!(Value::from(SomeStatus::Failure), Value::Int(2));
}
