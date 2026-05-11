use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod test;

/// Attribute macro for test functions that use parameters
/// that implement [`TestHarness`] trait.
///
/// Function should be `async`.
///
/// For each parameter, a new item will be created using `TestHarness::initialize`,
/// and the function will be called with the resulting values.
///
/// ```ignore
/// #[test]
/// async fn my_test(db: Database, mq: Queue) {
///    // use `db`and `mq`
/// }
///
/// #[async_trait]
/// impl TestHarness for Database {
///     type Error = DatabaseError;
///     async fn initialize() -> Result<Self, DatabaseError> {
///         // connect to DB and prepare data
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, test_func: TokenStream) -> TokenStream {
    let test_func = parse_macro_input!(test_func as ItemFn);
    test::run(attr.into(), test_func).into()
}
