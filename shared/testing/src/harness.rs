use std::{path::Path, sync::Arc};

use asez2_shared_db::result::SharedDbError;
use sqlx::PgPool;

use crate::{
    db::{apply_sql, prepare_for_test},
    id_pool::FreeId,
};

/// Ресурс, который умеет инициализировать себя для теста.
///
/// Два метода — потому что иногда нужен чистый ресурс (`initialize`),
/// а иногда — с предзагруженными данными из SQL-фикстуры (`initialize_with`).
/// Макрос `#[test]` вызывает нужный вариант в зависимости от того,
/// есть ли у параметра значение.
#[async_trait::async_trait]
pub trait TestHarness: Sized {
    type Error;
    type Arg;

    async fn initialize() -> Result<Self, Self::Error>;
    async fn initialize_with(arg: Self::Arg) -> Result<Self, Self::Error>;
}

/// Пул соединений к изолированной тестовой БД.
///
/// `_id: FreeId` — суффикс БД. Живёт столько же сколько `TestDbPool`:
/// пока тест работает, БД занята. При дропе суффикс возвращается в пул.
///
/// Параллельные тесты получают разные `FreeId` → разные БД → нет конкурентных
/// изменений между тестами. Первый тест на каждой БД применяет миграции,
/// последующие только TRUNCATE — за счёт флага `FreeId::is_new()`.
///
/// `PgPool` может "течь" (соединения не закрываются автоматически при дропе пула
/// в sqlx 0.5). Для тестов это приемлемо — процесс завершается после прогона.
pub struct TestDbPool {
    pool: Arc<PgPool>,
    _id: FreeId,
}

impl TestDbPool {
    pub async fn new() -> Result<Self, SharedDbError> {
        let (pool, _id) = prepare_for_test(Some(Path::new("migrations"))).await?;

        Ok(TestDbPool { pool, _id })
    }

    pub async fn apply_fixture(
        &self,
        fixture: &'static str,
    ) -> Result<(), TestDbError> {
        apply_sql(&self.pool, fixture).await
    }
}

/// Алиас для удобства в тестах — чтобы не импортировать sqlx напрямую.
pub type DbPool = PgPool;
pub type TestDbError = SharedDbError;

#[async_trait::async_trait]
impl TestHarness for TestDbPool {
    type Error = TestDbError;
    type Arg = &'static str;

    async fn initialize() -> Result<Self, Self::Error> {
        TestDbPool::new().await
    }

    async fn initialize_with(fixture: &'static str) -> Result<Self, Self::Error> {
        let this = TestDbPool::new().await?;
        this.apply_fixture(fixture).await?;
        Ok(this)
    }
}

impl std::ops::Deref for TestDbPool {
    type Target = Arc<PgPool>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

#[macro_export]
macro_rules! test_db_pool {
    ($name:ident, $fixture:expr) => {
        struct $name($crate::harness::TestDbPool);

        #[async_trait::async_trait]
        impl $crate::harness::TestHarness for $name {
            type Error = $crate::harness::TestDbError;
            type Arg = &'static str;

            async fn initialize() -> Result<Self, Self::Error> {
                let this = $crate::harness::TestDbPool::new().await?;
                this.apply_fixture($fixture).await?;
                Ok($name(this))
            }

            async fn initialize_with(
                fixture: &'static str,
            ) -> Result<Self, Self::Error> {
                let this = $crate::harness::TestDbPool::new().await?;
                this.apply_fixture(fixture).await?;
                Ok($name(this))
            }
        }

        impl std::ops::Deref for $name {
            type Target = std::sync::Arc<$crate::harness::DbPool>;

            fn deref(&self) -> &Self::Target {
                self.0.deref()
            }
        }
    };
}
