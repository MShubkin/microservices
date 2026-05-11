# FieldnameAccess Derive Macro

A derive macro for reading and writing struct fields by name at runtime — useful when the field name comes from data rather than being hardcoded.

### Container Attributes

- `#[fieldname_enum(name = "NewName")]` — name of the generated enum containing all possible field variants.

```rust
use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess, Default)]
#[fieldname_enum(name = "NewName")]
struct NamedFieldname {
    name: String,
    age: i64,
}

let mut instance = NamedFieldname::default();
match instance.field("name").unwrap() {
    NewName::String(val) => {}
    NewName::I64(val) => {}
}
match instance.field_mut("name").unwrap() {
    NewNameMut::String(val) => {}
    NewNameMut::I64(val) => {}
}
```

- `#[fieldname_enum(derive = [Debug, Clone], derive_mut = [Debug])]` — derive macros for the generated enums. `derive` applies to the immutable reference enum, `derive_mut` to the mutable reference enum. Useful when you need to clone a `&mut` reference.

```rust
use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess)]
#[fieldname_enum(derive = [Debug, Clone], derive_mut = [Debug])]
struct NamedFieldname {
    name: String,
    age: i64,
}
```

- `#[fieldname_enum(derive_all = [Debug])]` — applies the same derive macros to both the immutable and mutable enums.

```rust
use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess)]
#[fieldname_enum(derive_all = [Debug])]
struct NamedFieldname {
    name: String,
    age: i64,
}
```

### Field Attributes

- `#[fieldname = "AmazingAge"]` — overrides the enum variant name for this field.

```rust
use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess, Default)]
struct NamedFieldname {
    name: String,
    #[fieldname = "AmazingAge"]
    age: i64,
}

let mut instance = NamedFieldname::default();
match instance.field("name").unwrap() {
    NamedFieldnameField::String(val) => {}
    NamedFieldnameField::AmazingAge(val) => {}
}
match instance.field_mut("name").unwrap() {
    NamedFieldnameFieldMut::String(val) => {}
    NamedFieldnameFieldMut::AmazingAge(val) => {}
}
```

### Practical Example

Given a `User` struct and a `Crit` (criterion), you can determine the next action dynamically:

```rust
use fieldname_access::FieldnameAccess;

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
    name: String::from("Vasya"),
    does_love_flowers: true,
};

let crits = vec![
    Crit { value: String::from("Va"), field: String::from("name"), kind: CritKind::Contains },
    Crit { value: String::from("true"), field: String::from("does_love_flowers"), kind: CritKind::Equals },
    Crit { value: String::from("18"), field: String::from("age"), kind: CritKind::BiggerThan },
];

let its_ok = crits.into_iter().all(|crit| {
    let user_field = user.field(&crit.field).expect("criterion has an invalid field name");
    match crit.kind {
        CritKind::Contains => match user_field {
            UserField::String(str) => str.contains(&crit.value),
            _ => panic!("criterion has an invalid value"),
        },
        CritKind::Equals => match user_field {
            UserField::String(str) => str.eq(&crit.value),
            UserField::U64(int) => int.eq(&crit.value.parse::<u64>().unwrap()),
            UserField::Bool(boolean) => boolean.eq(&crit.value.parse::<bool>().unwrap()),
        },
        CritKind::BiggerThan => match user_field {
            UserField::U64(int) => int > &crit.value.parse::<u64>().unwrap(),
            _ => panic!("criterion has an invalid value"),
        },
    }
});
assert!(its_ok);
```

Fields can also be mutated:

```rust
if let Some(UserFieldMut::Bool(does_love_flowers)) = user.field_mut("does_love_flowers") {
    *does_love_flowers = false;
}
assert!(!user.does_love_flowers);
```

And you can iterate over all fields:

```rust
let output = user
    .field_iter()
    .map(|(field_name, enum_var)| format!("{}={:?}", field_name, enum_var))
    .join("\n");
println!("{}", output);
```
