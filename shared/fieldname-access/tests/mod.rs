use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess)]
struct TestStruct {
    name: String,
    age: u8,
    additional_info: Option<AdditionalInfo>,
}

struct AdditionalInfo {
    does_love_flowers: bool,
}

#[test]
fn field_success_access() {
    let info = AdditionalInfo {
        does_love_flowers: true,
    };
    let test_struct = TestStruct {
        age: 7,
        name: String::from("Вася"),
        additional_info: Some(info),
    };

    let age = match test_struct.field("age") {
        Some(TestStructField::U8(age)) => age,
        _ => panic!("Провал"),
    };
    assert_eq!(age, &test_struct.age);

    let name = match test_struct.field("name") {
        Some(TestStructField::String(name)) => name,
        _ => panic!("Провал"),
    };
    assert_eq!(name, &test_struct.name);

    let info = match test_struct.field("additional_info") {
        Some(TestStructField::OptionAdditionalInfo(info)) => info,
        _ => panic!("Провал"),
    }
    .as_ref()
    .unwrap();
    assert_eq!(
        info.does_love_flowers,
        test_struct.additional_info.as_ref().unwrap().does_love_flowers
    );
}

#[test]
fn field_success_access_mut() {
    let info = AdditionalInfo {
        does_love_flowers: true,
    };
    let mut test_struct = TestStruct {
        age: 7,
        name: String::from("Вася"),
        additional_info: Some(info),
    };

    let age = match test_struct.field_mut("age") {
        Some(TestStructFieldMut::U8(age)) => age,
        _ => panic!("Провал"),
    };
    assert_eq!(age.clone(), test_struct.age);

    let name = match test_struct.field_mut("name") {
        Some(TestStructFieldMut::String(name)) => name,
        _ => panic!("Провал"),
    };
    assert_eq!(name.clone(), test_struct.name);

    let info = match test_struct.field_mut("additional_info") {
        Some(TestStructFieldMut::OptionAdditionalInfo(info)) => info,
        _ => panic!("Провал"),
    }
    .as_ref()
    .unwrap();
    assert_eq!(
        info.does_love_flowers.clone(),
        test_struct.additional_info.unwrap().does_love_flowers
    );
}

#[test]
fn failure_access() {
    let info = AdditionalInfo {
        does_love_flowers: true,
    };
    let mut test_struct = TestStruct {
        age: 7,
        name: String::from("Вася"),
        additional_info: Some(info),
    };

    assert!(test_struct.field("something").is_none());
    assert!(test_struct.field_mut("not_worthy").is_none());
}

#[test]
fn field_mutation() {
    let info = AdditionalInfo {
        does_love_flowers: true, // important
    };
    let mut test_struct = TestStruct {
        age: 7,
        name: String::from("Вася"),
        additional_info: Some(info),
    };

    let info = match test_struct.field_mut("additional_info") {
        Some(TestStructFieldMut::OptionAdditionalInfo(important)) => important,
        _ => panic!("Failed"),
    };
    *info = None;

    assert!(test_struct.additional_info.is_none());
}

#[test]
fn complex_test() {
    #[derive(FieldnameAccess)]
    struct User {
        name: String,
        age: u64,
        does_love_flowers: bool,
    }

    struct Crit {
        value: String,
        field: String,
        kind: CritKind,
    }

    enum CritKind {
        Contains,
        Equals,
        BiggerThan,
    }

    let mut user = User {
        age: 2022,
        name: String::from("Вася"),
        does_love_flowers: true,
    };

    let crits = vec![
        Crit {
            value: String::from("Ва"),
            field: String::from("name"),
            kind: CritKind::Contains,
        },
        Crit {
            value: String::from("true"),
            field: String::from("does_love_flowers"),
            kind: CritKind::Equals,
        },
        Crit {
            value: String::from("18"),
            field: String::from("age"),
            kind: CritKind::BiggerThan,
        },
    ];

    let its_ok = crits.into_iter().all(|crit| {
        let user_field =
            user.field(&crit.field).expect("Критерий имеет невалидное имя поля");
        match crit.kind {
            CritKind::Contains => match user_field {
                UserField::String(str) => str.contains(&crit.value),
                _ => panic!("Критерий имеет невалидное значение"),
            },
            CritKind::Equals => match user_field {
                UserField::String(str) => str.eq(&crit.value),
                UserField::U64(int) => int.eq(&crit.value.parse::<u64>().unwrap()),
                UserField::Bool(boolean) => {
                    boolean.eq(&crit.value.parse::<bool>().unwrap())
                }
                UserField::None => panic!("You're not going to hit this here."),
            },
            CritKind::BiggerThan => match user_field {
                UserField::U64(int) => int > &crit.value.parse::<u64>().unwrap(),
                _ => panic!("Критерий имеет невалидное значение"),
            },
        }
    });
    assert!(its_ok);

    // Также можно изменять поля
    if let Some(UserFieldMut::Bool(does_love_flowers)) =
        user.field_mut("does_love_flowers")
    {
        *does_love_flowers = false;
    }
    assert!(!user.does_love_flowers);
}

#[derive(FieldnameAccess)]
struct TestComplexPath {
    name: std::option::Option<String>,
    age: std::option::Option<std::option::Option<i64>>,
}

#[test]
fn test_complex_type_path() {
    let structure = TestComplexPath {
        name: Some(String::from("Вася")),
        age: Some(Some(321)),
    };

    if let Some(TestComplexPathField::OptionString(Some(val))) =
        structure.field("name")
    {
        assert_eq!(val, &"Вася");
    } else {
        panic!("Провал");
    }

    if let Some(TestComplexPathField::OptionOptionI64(Some(Some(val)))) =
        structure.field("age")
    {
        assert_eq!(val, &321);
    } else {
        panic!("Провал");
    }
}

#[derive(FieldnameAccess, Clone, Copy)]
struct GenericStruct<'a, T, F>
where
    T: Into<String>,
{
    name: &'a T,
    age: F,
}

#[test]
fn generic_struct_fieldname_access() {
    let structure = GenericStruct {
        age: 123,
        name: &String::from("123"),
    };

    match structure.field("name").unwrap() {
        GenericStructField::T(name) => assert_eq!(*name, "123"),
        GenericStructField::F(_age) => panic!("Expected T not F"),
        GenericStructField::None => panic!("You're not going to hit this here."),
    }
}

#[derive(FieldnameAccess)]
#[fieldname_enum(name = "Amazingly", derive = [Debug, Clone], derive_mut = [Debug])]
struct NamedFieldname {
    name: String,
    #[fieldname = "MyAge"]
    age: i64,
    age_of_dog: i64,
    age_of_cat: i64,
}

#[derive(FieldnameAccess)]
#[fieldname_enum(name = "AmazinglyTwo", derive_all = [Debug])]
#[allow(unused)]
struct NamedFieldnameTwo {
    name: String,
    #[fieldname = "MyAge"]
    age: i64,
}

#[test]
fn attributes() {
    let mut structure = NamedFieldname {
        age: 123,
        name: String::from("123"),
        age_of_dog: 123,
        age_of_cat: 123,
    };
    match structure.field("name").unwrap() {
        Amazingly::String(val) => {
            let val_clone = val.clone();
            assert_eq!(val_clone, "123")
        }
        Amazingly::MyAge(val) => assert_eq!(*val, 123),
        Amazingly::I64(val) => assert_eq!(*val, 123),
        Amazingly::None => panic!("You're not going to hit this here."),
    }
    match structure.field_mut("name").unwrap() {
        AmazinglyMut::String(val) => {
            println!("{}", val);
            assert_eq!(val, "123")
        }
        AmazinglyMut::MyAge(val) => assert_eq!(*val, 123),
        AmazinglyMut::I64(val) => assert_eq!(*val, 123),
        AmazinglyMut::None => panic!("You're not going to hit this here."),
    }
}
