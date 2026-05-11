use format_tools::fomat;

#[test]
fn basic_fomat() {
    let res = fomat!(
        "[-",
        "-]",
        "Hello, [-name-]. Have a good day {hopefully}",
        name = "John"
    );

    assert_eq!(res, "Hello, John. Have a good day {hopefully}");
}
