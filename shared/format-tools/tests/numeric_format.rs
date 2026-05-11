use format_tools::numeric_format;

#[test]
fn general_usage() {
    let count = 123;
    let another_var = 123;
    let name = "ПД";
    let res = numeric_format!("{} {@маршрут|маршрута|маршрутов} {name} с типом {ty} удалены", another_var, @count, ty = 2);

    assert_eq!(res, "123 маршрута ПД с типом 2 удалены");
}

#[test]
fn direct_numeric_param() {
    let count = 123;
    let name = "ПД";
    let res = numeric_format!(
        "{@count} {@маршрут|маршрута|маршрутов} {name} с типом {ty} удалены",
        ty = 2
    );
    assert_eq!(res, "123 маршрута ПД с типом 2 удалены");
}

#[test]
fn controlled_relation_suffix() {
    let test: &[(&[usize], &str)] = &[
        (&[1, 121], "маршрут"),
        (&[4, 24], "маршрута"),
        (&[5, 11, 25], "маршрутов"),
    ];

    test.iter().for_each(|(cases, exp)| {
        cases.iter().for_each(|&case| {
            assert_eq!(exp, &numeric_format!("маршрут{@|а|ов}", @case))
        })
    })
}

#[test]
fn controlled_relation_full() {
    let test: &[(&[usize], &str)] =
        &[(&[1, 121], "пчела"), (&[4, 24], "пчелы"), (&[5, 11, 25], "пчел")];

    test.iter().for_each(|(cases, exp)| {
        cases.iter().for_each(|&case| {
            assert_eq!(exp, &numeric_format!("{@пчела|пчелы|пчел}", @case))
        })
    })
}

#[test]
fn conformed_relation() {
    let test: &[(&[usize], &str)] =
        &[(&[1, 121], "остановлен"), (&[11, 24], "остановлены")];

    test.iter().for_each(|(cases, exp)| {
        cases.iter().for_each(|&case| {
            assert_eq!(exp, &numeric_format!("остановлен{@|ы}", @case))
        })
    })
}
