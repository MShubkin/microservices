use std::convert::Infallible;

use testing::TestHarness;

struct Foo;

#[async_trait::async_trait]
impl TestHarness for Foo {
    type Error = Infallible;
    type Arg = ();

    async fn initialize() -> Result<Self, Self::Error> {
        Ok(Foo)
    }

    async fn initialize_with(_: ()) -> Result<Self, Self::Error> {
        Ok(Foo)
    }
}

#[testing::test]
async fn my_test_with_db_pool(_harness: Foo) {}
