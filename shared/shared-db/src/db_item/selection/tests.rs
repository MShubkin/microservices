use super::*;

macro_rules! derive_tests {
    ($($aggr_arrays:ident)?) => {
mod animal_tests {
    use serde::{Serialize, Deserialize};
    use crate::db_item::int_array::AsezArray;
    use shared_db_derive::{DbAdaptor, DbItem};

    use super::*;
    use crate as asez2_shared_db;

    #[derive(Debug, Clone, DbItem, DbAdaptor)]
    #[adaptor_derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
    #[item_table = "animals"]
    $(#[$aggr_arrays])?
    pub struct Animal {
        #[item_field_pkey]
        id: i64,
        #[adaptor_attributes(#[serde(default)])]
        name: String,
        weight_g: i32,
        habitat: Option<String>,
        functionality_id_list: Option<AsezArray<i16>>,
    }

    #[tokio::test]
    async fn test_array_overlaps_filter() {
        use crate::db_item::selection::SelectMaker;
        use crate::db_item::Select;
        use crate::value::Value;
        use crate::db_item::int_array::AsezArray;

        let select = Select::with_fields(["id", "name", "functionality_id_list"])
            .array_overlaps(
                "functionality_id_list",
                Value::Vec16(AsezArray(vec![1, 2])),
            );

        let ok = SelectMaker::<Animal>::check_select(&select);
        assert!(ok.is_ok());

        let s = SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();

        assert_eq!(
            s.query_string(),
            "SELECT id,name,functionality_id_list FROM animals WHERE functionality_id_list &&$1"
        );
    }

    #[tokio::test]
    async fn test_array_overlaps_with_other_filters() {
        let select = Select::with_fields(["id", "name", "functionality_id_list"])
            .array_overlaps(
                "functionality_id_list",
                Value::Vec16(AsezArray(vec![1, 2])),
            )
           .eq("name", "test")
           .eq("weight_g", 100);

        let s = SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();

        assert!(
            s.query_string().contains("functionality_id_list &&$1"),
            "SQL: {}",
            s.query_string()
        );
        assert!(s.query_string().contains("name =$2"));
        assert!(s.query_string().contains("weight_g =$3"));
    }

    #[test]
    fn test_check_select_1() {
        let s = Select::with_fields(["id", "name", "weight_g", "habitat"]);
        let ok = SelectMaker::<Animal>::check_select(&s);
        assert!(ok.is_ok());
    }

    #[test]
    fn test_check_bad_select_1() {
        let s = Select::with_fields(["id", "select", "*", "where", "uuid=52"]);
        let err = SelectMaker::<Animal>::check_select(&s);
        assert!(err.is_err());
        assert_eq!(
            &err.unwrap_err().to_string(),
            "Field `select` not in table `animals`");

        // Again with normal creator function.
        let s = Select::with_fields(["id", "select", "*", "where", "uuid=52"]);
        let err = SelectMaker::<Animal>::check_select(&s);
        assert!(err.is_err());
        assert_eq!(
            &err.unwrap_err().to_string(),
            "Field `select` not in table `animals`");
    }

    #[test]
    fn test_check_bad_select_3() {
        let filters = Filter::in_any::<_, Value>("jimmy tables", []);
        let s = Select {
            field_list: vec!["id".to_string()],
            filter_list: filters.into(),
            ..Default::default()
        };
        let err = SelectMaker::<Animal>::check_select(&s);
        assert!(err.is_err());
        assert_eq!(
            &err.unwrap_err().to_string(),
            "Filter field `jimmy tables` not in table `animals`");
    }

    #[test]
    fn test_check_bad_select_4() {
        let order_list = vec![
            FieldSortOrder {
                field: "update".to_string(),
                order: FieldSortKind::Asc,
                null_position: None
            },
            FieldSortOrder {
                field: "animals".to_string(),
                order: FieldSortKind::Desc,
                null_position: None
            }
        ];
        let s = Select {
            field_list: vec!["id".to_string()],
            order_list,
            ..Default::default()
        };
        let err = SelectMaker::<Animal>::check_select(&s);
        assert!(err.is_err());
        assert_eq!(
            &err.unwrap_err().to_string(),
            "Order key `update` not in table `animals`");
    }

    #[tokio::test]
    async fn test_check_select_with_null_ok() {
        // С `SelectionKind::Equals` и несколькими значениями
        let select = Select::with_fields(["id", "name", "weight_g", "habitat"]).add_expand_filter(
            "habitat",
            SelectionKind::Equals,
            vec![Value::Null, Value::String(String::from("1")), Value::String(String::from("2"))],
        );
        let check = SelectMaker::<Animal>::check_select(&select);
        assert!(check.is_ok());
        let s = SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await;
        // NB: Strangely unwrap_err causes problems.
        let s = match s {
            Ok(_) => panic!(),
            Err(e) => e.to_string(),
        };
        assert_eq!(
            &s,
            "`=` не поддерьивает больше одного значение. Найдено 3. (Фильтр на habitat: [Null, String(\"1\"), String(\"2\")])"
        );

      // С `SelectionKind::Equals` и одним значением
      let select = Select::with_fields(["id", "name", "weight_g", "habitat"]).add_expand_filter(
        "habitat",
        SelectionKind::Equals,
        vec![Value::Null],
      );
      let check = SelectMaker::<Animal>::check_select(&select);
      assert!(check.is_ok());
      let s = SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
      assert_eq!(
          s.query_string(),
          "SELECT id,name,weight_g,habitat FROM animals WHERE habitat IS NULL"
      );

      // С `SelectionKind::NotEquals` и несколькими значениями
      let select = Select::with_fields(["id", "name", "weight_g", "habitat"]).add_expand_filter(
        "habitat",
        SelectionKind::NotIn,
        vec![Value::Null, Value::String(String::from("1")), Value::String(String::from("2"))],
      );
      let check = SelectMaker::<Animal>::check_select(&select);
      assert!(check.is_ok());
      let s = SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
      assert_eq!(
          s.query_string(),
          "SELECT id,name,weight_g,habitat FROM animals WHERE NOT ((habitat=ANY($1) OR habitat IS NOT NULL))"
      );

      // С `SelectionKind::NotEquals` и без значений
      let select = Select::with_fields(["id", "name", "weight_g", "habitat"]).add_expand_filter(
        "habitat",
        SelectionKind::NotEquals,
        vec![Value::Null],
      );
      let check = SelectMaker::<Animal>::check_select(&select);
      assert!(check.is_ok());
      let s = SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
      assert_eq!(
          s.query_string(),
          "SELECT id,name,weight_g,habitat FROM animals WHERE habitat IS NOT NULL"
      );

      // С `SelectionKind::In` и с несколькими значениями
      let select = Select::with_fields(["id", "name", "weight_g", "habitat"]).add_expand_filter(
        "habitat",
        SelectionKind::In,
        vec![Value::Null, Value::String(String::from("1")), Value::String(String::from("2"))],
      );
      let check = SelectMaker::<Animal>::check_select(&select);
      assert!(check.is_ok());
      let s = SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
      assert_eq!(
          s.query_string(),
          "SELECT id,name,weight_g,habitat FROM animals WHERE (habitat=ANY($1) OR habitat IS NULL)"
      );

      // С `SelectionKind::In` и без значений
      let select = Select::with_fields(["id", "name", "weight_g", "habitat"]).add_expand_filter(
        "habitat",
        SelectionKind::In,
        vec![Value::Null],
      );
      let check = SelectMaker::<Animal>::check_select(&select);
      assert!(check.is_ok());
      let s = SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
      assert_eq!(
          s.query_string(),
          "SELECT id,name,weight_g,habitat FROM animals WHERE habitat IS NULL"
      );

      // С `SelectionKind::NotIn` и с несколькими значениями
      let select = Select::with_fields(["id", "name", "weight_g", "habitat"]).add_expand_filter(
        "habitat",
        SelectionKind::NotIn,
        vec![Value::Null, Value::String(String::from("1")), Value::String(String::from("2"))],
      );
      let check = SelectMaker::<Animal>::check_select(&select);
      assert!(check.is_ok());
      let s = SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
      assert_eq!(
          s.query_string(),
          "SELECT id,name,weight_g,habitat FROM animals WHERE NOT ((habitat=ANY($1) OR habitat IS NOT NULL))"
      );

      // С `SelectionKind::NotIn` и без значений
      let select = Select::with_fields(["id", "name", "weight_g", "habitat"]).add_expand_filter(
        "habitat",
        SelectionKind::NotIn,
        vec![Value::Null],
      );
      let check = SelectMaker::<Animal>::check_select(&select);
      assert!(check.is_ok());
      let s = SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
      assert_eq!(
          s.query_string(),
          "SELECT id,name,weight_g,habitat FROM animals WHERE NOT (habitat IS NOT NULL)"
      );

      // С `SelectionKind::Contains`, что является ошибкой
      let select = Select::with_fields(["id", "name", "weight_g", "habitat"]).add_expand_filter(
        "habitat",
        SelectionKind::Contains,
        vec![Value::Null],
      );
      let check = SelectMaker::<Animal>::check_select(&select);
      assert!(check.is_ok());
      assert!(SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .is_err());

      // С `SelectionKind::GreaterEqual`, что является ошибкой
      let select = Select::with_fields(["id", "name", "weight_g", "habitat"]).add_expand_filter(
        "habitat",
        SelectionKind::GreaterEqual,
        vec![Value::Null],
      );
      let check = SelectMaker::<Animal>::check_select(&select);
      assert!(check.is_ok());
      assert!(SelectMaker::<Animal>::start(&select)
            .await
            .unwrap()
            .stack()
            .await
            .is_err());
    }


    #[tokio::test]
    async fn test_questionable_select_1() {
        let field_list = ["id", "name", "weight_g", "habitat"]
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        let filter_list = Filter::not_eq("weight_g", "delete from animals")
            .into();

        let s = Select {
            field_list,
            filter_list,
            ..Default::default()
        };
        let ok = SelectMaker::<Animal>::check_select(&s);
        assert!(ok.is_ok());
        let s = SelectMaker::<Animal>::start(&s)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
        assert_eq!(
            s.query_string(),
            "SELECT id,name,weight_g,habitat FROM animals WHERE weight_g !=$1"
        );
    }

    #[tokio::test]
    async fn test_order_select_1() {
        let s = Select::with_fields(["id", "name", "weight_g", "habitat"])
            .add_expand_filter(
                "weight_g",
                SelectionKind::NotEquals,
                vec![Value::String("delete from animals".to_string())],
            )
            .add_replace_order("weight_g", FieldSortKind::Asc);

        let ok = SelectMaker::<Animal>::check_select(&s);
        assert!(ok.is_ok());
        let s = SelectMaker::<Animal>::start(&s)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
        assert_eq!(
            s.query_string(),
            "SELECT id,name,weight_g,habitat FROM animals WHERE weight_g !=$1 ORDER BY weight_g ASC"
        );
    }

    #[tokio::test]
    async fn order_select_with_null_possition() {
        let s = Select::with_fields(["id", "name", "weight_g", "habitat"])
            .add_expand_filter(
                "weight_g",
                SelectionKind::NotEquals,
                vec![Value::String("delete from animals".to_string())],
            )
            .add_replace_order("weight_g", FieldSortKind::Asc)
            .add_replace_order("habitat", FieldSortKind::Desc)
            .with_null_position(NullPosition::First);

        let ok = SelectMaker::<Animal>::check_select(&s);
        assert!(ok.is_ok());
        let s = SelectMaker::<Animal>::start(&s)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
        assert_eq!(
            s.query_string(),
            "SELECT id,name,weight_g,habitat FROM animals WHERE weight_g !=$1 ORDER BY weight_g ASC NULLS FIRST, habitat DESC NULLS FIRST"
        );
    }

    #[tokio::test]
    async fn test_order_select_first() {
        let s = Select::with_fields::<&str, _>([])
            .add_replace_order("weight_g", FieldSortKind::Desc)
            .take_first()
            .add_expand_filter(
                "weight_g",
                SelectionKind::NotEquals,
                vec![Value::String("delete from animals".to_string())],
            );

        let ok = SelectMaker::<Animal>::check_select(&s);
        assert!(ok.is_ok());
        let s = SelectMaker::<Animal>::start(&s)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
        assert_eq!(
            s.query_string(),
            "SELECT * FROM animals WHERE weight_g !=$1 ORDER BY weight_g DESC FETCH FIRST ROW ONLY"
        );
    }

    #[tokio::test]
    async fn test_order_distinct_select() {
        let s = Select::with_fields::<&str, _>([])
            .add_replace_order("id", FieldSortKind::Desc)
            .add_replace_order("weight_g", FieldSortKind::Desc)
            .take_first()
            .add_expand_filter(
                "weight_g",
                SelectionKind::NotEquals,
                vec![Value::String("delete from animals".to_string())],
            )
            .distinct_on(&["id", "weight_g"]);

        let ok = SelectMaker::<Animal>::check_select(&s);
        assert!(ok.is_ok());
        let s = SelectMaker::<Animal>::start(&s)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
        assert_eq!(
            s.query_string(),
            "SELECT DISTINCT ON(id,weight_g) * FROM animals WHERE weight_g !=$1 ORDER BY id DESC, weight_g DESC FETCH FIRST ROW ONLY"
        );
    }

    #[tokio::test]
    async fn test_multiple_filters_for_one_field() {
        let s = Select::with_fields::<&str, _>([])
            .add_replace_order("id", FieldSortKind::Desc)
            .add_replace_order("weight_g", FieldSortKind::Desc)
            .take_first()
            .add_expand_filter(
                "id",
                SelectionKind::NotEquals,
                [50],
            )
            .add_expand_filter(
                "id",
                SelectionKind::Between,
                [5, 100],
            )
            .add_expand_filter(
                "weight_g",
                SelectionKind::NotEquals,
                ["delete from animals"],
            )
            .distinct_on(&["id", "weight_g"]);

        let ok = SelectMaker::<Animal>::check_select(&s);
        assert!(ok.is_ok());
        let s = SelectMaker::<Animal>::start(&s)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
        assert_eq!(
            s.query_string(),
            "SELECT DISTINCT ON(id,weight_g) * FROM animals WHERE ( id !=$1 AND id BETWEEN $2 AND $3 AND weight_g !=$4) ORDER BY id DESC, weight_g DESC FETCH FIRST ROW ONLY"
        );
    }

    #[tokio::test]
    async fn test_take_n_skip_n() {
        let s = Select::default()
            // test that we replace first with take_n
            .take_first()
            .take_n(55)
            .offset(234);

        let ok = SelectMaker::<Animal>::check_select(&s);
        assert!(ok.is_ok());
        let s = SelectMaker::<Animal>::start(&s)
            .await
            .unwrap()
            .stack()
            .await
            .unwrap();
        assert_eq!(
            s.query_string(),
            "SELECT * FROM animals OFFSET 234 FETCH NEXT 55 ROW ONLY"
        );
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

mod filter_tree {
    use crate::db_item::selection::filters::*;
    use crate::Value;

    #[test]
    fn test_simplest_filter_tree() {
        let f: FilterTree = Filter {
            field: "id".to_string(),
            kind: SelectionKind::In,
            values: vec![Value::Int(35), Value::Int(93), Value::Int(-3)],
        }
        .into();

        assert_eq!(f, FilterTree::Filter(Filter::in_any("id", vec![35, 93, -3],)));

        let mut sql = String::new();
        let n = f.build_sql(&mut sql, 1).unwrap();

        assert_eq!(n, 2);
        assert_eq!(&sql, " id=ANY($1)")
    }

    #[test]
    fn test_simple_and_filter_tree() {
        let f1 = Filter::in_any("id", [35, 93, -3]);
        let f2 = Filter::in_any("name", ["bob", "amy"]);
        let f = FilterTree::and_from_list(vec![f1, f2]);

        let mut sql = String::new();
        let n = f.build_sql(&mut sql, 1).unwrap();

        assert_eq!(n, 3);
        assert_eq!(&sql, " ( id=ANY($1) AND name=ANY($2))")
    }

    #[test]
    fn test_simple_or_filter_tree() {
        let f1 = Filter::in_any("id", [35, 93, -3]);
        let f2 = Filter::in_any("name", ["bob", "amy"]);
        let f3 = Filter::in_any("kind", [Value::Null]);
        let f = FilterTree::or_from_list(vec![f1, f2, f3]);

        let mut sql = String::new();
        let n = f.build_sql(&mut sql, 1).unwrap();

        assert_eq!(n, 3);
        assert_eq!(&sql, " ( id=ANY($1) OR name=ANY($2) OR kind IS NULL)")
    }

    #[test]
    fn test_and_and_or() {
        let f1 = Filter::in_any("id", [35, 93, -3]);
        let f2 = Filter::in_any("name", ["bob", "amy"]);
        let and_tree = FilterTree::and_from_list(vec![f1, f2]);

        let f1 = Filter::in_any("id", [35, 93, -3]);
        let f2 = Filter::in_any("name", ["bob", "amy"]);
        let f3 = Filter::in_any("kind", [Value::Null]);
        let or_tree = FilterTree::or_from_list(vec![f1, f2, f3]);

        let full_tree = and_tree.and(or_tree);

        let mut sql = String::new();
        let n = full_tree.build_sql(&mut sql, 1).unwrap();

        assert_eq!(n, 5, "{}", sql);
        assert_eq!(
            &sql,
            " ( \
( id=ANY($1) AND name=ANY($2)) \
AND \
( id=ANY($3) OR name=ANY($4) OR kind IS NULL))"
        );
    }

    #[test]
    fn test_and_or_or() {
        let f1 = Filter::in_any("id", [35, 93, -3]);
        let f2 = Filter::in_any("name", ["bob", "amy"]);
        let and_tree = FilterTree::and_from_list(vec![f1, f2]);

        let f1 = Filter::in_any("id", [35, 93, -3]);
        let f2 = Filter::in_any("name", ["bob", "amy"]);
        let f3 = Filter::in_any("kind", [1.into(), 2.into(), Value::Null]);
        let or_tree = FilterTree::or_from_list(vec![f1, f2, f3]);

        let full_tree = and_tree.or(or_tree);

        let mut sql = String::new();
        let n = full_tree.build_sql(&mut sql, 1).unwrap();

        assert_eq!(n, 6);
        assert_eq!(
            &sql,
            " ( \
( id=ANY($1) AND name=ANY($2)) \
OR \
( id=ANY($3) OR name=ANY($4) OR (kind=ANY($5) OR kind IS NULL)))"
        );
    }

    #[test]
    fn test_complex() {
        let f1 = Filter::in_any("id", [35, 93, -3]);
        let f2 = Filter::in_any("name", ["bob", "amy"]);
        let and_tree = FilterTree::and_from_list(vec![f1, f2]);

        let f1 = Filter::in_any("id", [35, 93, -3]);
        let f2 = Filter::in_any("name", ["bob", "amy"]);
        let f3 = Filter::in_any("kind", [Value::Null]);
        let or_tree = FilterTree::or_from_list(vec![f1, f2, f3]);

        let f1 = Filter::eq("colour", "green");
        let f2 = Filter::eq("colour", "blue");
        let f3 = Filter::eq("colour", "yellow");
        let or_tree2 = FilterTree::or_from_list(vec![f1, f2, f3]);

        let f1 = Filter::eq("has_wheels", true);
        let f2 = Filter::eq("has_wings", true);
        let or_tree3 = FilterTree::or_from_list(vec![f1, f2]);

        let f1 = Filter::between("price", 1, 1000);
        let f2 = Filter::not_contains("flavour", "strawberry");
        let and_tree2 = FilterTree::and_from_list(vec![f1, f2]);

        let final_tree =
            and_tree2.or(or_tree2.or(or_tree3).or(and_tree.and(or_tree)));

        let mut sql = String::new();
        let n = final_tree.build_sql(&mut sql, 1).unwrap();

        assert_eq!(n, 13);
        assert_eq!(
            &sql,
            " ( \
( price BETWEEN $1 AND $2 AND flavour !~$3) \
OR \
( ( ( colour =$4 OR colour =$5 OR colour =$6) \
OR \
( has_wheels =$7 OR has_wings =$8)) \
OR \
( ( id=ANY($9) AND name=ANY($10)) \
AND \
( id=ANY($11) OR name=ANY($12) OR kind IS NULL))))"
        );
    }

    #[derive(Debug)]
    struct TestFilter(SelectionKind, Vec<i32>);
    impl FilterTrait for TestFilter {
        type ValueType = i32;

        fn kind(&self) -> SelectionKind {
            self.0
        }

        fn values(&self) -> &[Self::ValueType] {
            &self.1
        }
    }

    #[test]
    fn to_jsonpath_filter() {
        for (test_filter, exp_value) in [
            (
                TestFilter(SelectionKind::Equals, vec![0]),
                "$.outer_field ? (@.inner_field == 0)",
            ),
            (
                TestFilter(SelectionKind::GreaterEqual, vec![0]),
                "$.outer_field ? (@.inner_field >= 0)",
            ),
            (
                TestFilter(SelectionKind::Between, vec![0, 10]),
                "$.outer_field ? (0 <= @.inner_field && @.inner_field <= 10)",
            ),
            (
                TestFilter(SelectionKind::In, vec![0, 1, 2]),
                "$.outer_field ? (@.inner_field == 0 || @.inner_field == 1 || @.inner_field == 2)",
            ),
            (
                TestFilter(SelectionKind::NotIn, vec![0, 1, 2]),
                "$.outer_field ? (@.inner_field != 0 && @.inner_field != 1 && @.inner_field != 2)",
            ),
        ] {
            let filter = convert_to_jsonpath(
                "db_field",
                "$.outer_field",
                "inner_field",
                &test_filter,
            )
            .expect("no error");

            assert_eq!(&filter.field, "db_field");
            assert_eq!(filter.kind, SelectionKind::Jsonpath);
            assert_eq!(&filter.values, &[Value::from(exp_value)]);
        }
    }

    #[test]
    fn to_jsonpath_filters() {
        let actual_filter = convert_jsonpath_filters(
            "db_field",
            "$.outer_field",
            "inner_field",
            &[
                TestFilter(SelectionKind::Equals, vec![0]),
                TestFilter(SelectionKind::In, vec![10, 20, 30]),
            ],
        )
        .expect("no error");

        let expected_filter = &[
            Filter {
                field: "db_field".to_string(),
                kind: SelectionKind::Jsonpath,
                values: vec![Value::from("$.outer_field ? (@.inner_field == 0)")],
            },
            Filter {
                field: "db_field".to_string(),
                kind: SelectionKind::Jsonpath,
                values: vec![Value::from("$.outer_field ? (@.inner_field == 10 || @.inner_field == 20 || @.inner_field == 30)")],
            },
        ];

        assert_eq!(&actual_filter, expected_filter);
    }

    #[test]
    fn single_value_filters() {
        assert_eq!(
            convert_filters_equal_to_value(&[TestFilter(
                SelectionKind::Equals,
                vec![10]
            )])
            .unwrap(),
            &10
        );
    }

    #[test]
    fn single_value_filters_errors() {
        let many_filters: &[TestFilter] = &[
            TestFilter(SelectionKind::Equals, vec![0]),
            TestFilter(SelectionKind::Equals, vec![1]),
        ];
        let no_values: &[TestFilter] = &[TestFilter(SelectionKind::Equals, vec![])];
        let many_values: &[TestFilter] =
            &[TestFilter(SelectionKind::Equals, vec![0, 1])];
        let invalid_kind: &[TestFilter] =
            &[TestFilter(SelectionKind::NotEquals, vec![1])];

        for fs in [many_filters, no_values, many_values, invalid_kind] {
            assert!(convert_filters_equal_to_value(fs).is_err(), "{fs:?}");
        }
    }
}
