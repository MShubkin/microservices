use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod test;

/// Атрибут-макрос для тестовых функций с параметрами, реализующими `TestHarness`.
///
/// Проблема, которую решает макрос: интеграционные тесты требуют ресурсов (БД, очередь),
/// которые нужно инициализировать перед каждым тестом и освободить после. Стандартный
/// `#[tokio::test]` не умеет передавать параметры в тестовую функцию. Этот макрос
/// разворачивает `async fn my_test(db: Database)` в блок, который вызывает
/// `Database::initialize().await` и передаёт результат в тело теста.
///
/// Каждый параметр должен реализовывать `TestHarness` — за счёт этого тест описывает
/// только что именно нужно (DB? RabbitMQ?), а как это поднять — знает харнесс.
///
/// ```ignore
/// #[test]
/// async fn my_test(db: Database, mq: Queue) {
///    // использовать `db` и `mq`
/// }
///
/// #[async_trait]
/// impl TestHarness for Database {
///     type Error = DatabaseError;
///     async fn initialize() -> Result<Self, DatabaseError> {
///         // подключиться к тестовой БД и подготовить данные
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, test_func: TokenStream) -> TokenStream {
    let test_func = parse_macro_input!(test_func as ItemFn);
    test::run(attr.into(), test_func).into()
}
