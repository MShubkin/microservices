mod fomat;
mod numeric_format;

/// Обертка над `format!` для более гибкого форматирования строки
///
/// ## Использование
///
/// Можно задать отличные от `{` и `}` символы для форматирования. Например:
/// ```rust, ignore
/// let res = fomat!(
///     "[-",
///     "-]",
///     "Hello, [-name-]. Have a good day {hopefully}",
///     name = "John"
/// );
/// assert_eq!(res, "Hello, John. Have a good day {hopefully}");
/// ```
#[proc_macro]
pub fn fomat(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fomat::fomat_inner(inp)
}

/// Макрос для подбора правильной формы слова в соответствии с числительным.
/// Использует [`shared_essential::application::message::numeral_relation`]
///
/// ## Использование
///
/// ```rust, ignore
/// let count = 123;
/// let name = "ПД";
/// let res = numeric_format!("{@count} {@маршрут|маршрута|маршрутов} {name} с типом {ty} удалены", ty = 2);
/// assert_eq!(res, "123 маршрута ПД с типом 2 удалены");
/// ```
///
/// или же
///
/// ```rust, ignore
/// let count = 123;
/// let another_var = count;
/// let name = "ПД";
/// let res = numeric_format!("{} {@маршрут|маршрута|маршрутов} {name} с типом {ty} удалены", another_var, @count, ty = 2);
/// assert_eq!(res, "123 маршрута ПД с типом 2 удалены");
/// ```
///
/// При перечислении форматирующих параметров надо указать параметр с префиксом `@`, который
/// является числительным, на основании которого будет выбираться правильная форма слова
///
/// * `{@singular|within_2_and_4_in_the_end|default}` использует `get_controlled_numeric`.
/// * `{@singular|plural}` использует `get_conformed_numeric`.
#[proc_macro]
pub fn numeric_format(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    numeric_format::numeric_format_inner(input)
        .unwrap_or_else(|err| err.to_compile_error().into())
}
