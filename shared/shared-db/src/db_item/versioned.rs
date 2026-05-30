//! Поддержка версионирования сущностей.
//!
//! Механизм предназначен для сущностей, у которых нужно хранить историю
//! изменений. Каждая версия записывается в отдельную таблицу с тем же набором
//! полей, что и оригинальная, плюс поле `pricing_version: i16`.
use super::*;
pub use shared_db_derive::DbVersioned;

use ahash::AHashMap;
use sqlx::FromRow;

#[async_trait::async_trait]
/// Трейт для [`DbItem`]-сущностей с поддержкой версий.
///
/// Связанный тип `Versioned` -- это структура в версионной таблице.
/// Обычно генерируется макросом `#[derive(DbVersioned)]`.
///
/// ТОДО: Пока что логика insert и update почти такие же, так как надо
/// чтобы код срабатывал для новых версии, даже если insert.
pub trait DbVersioned: DbItem {
    type Versioned: DbItem;

    /// Конвертирует текущий элемент в версионную запись с заданным номером версии.
    fn to_versioned(&self, pricing_version: i16) -> Self::Versioned;

    /// Конвертирует версионную запись обратно в активный элемент.
    fn to_active(v: &Self::Versioned) -> Self;

    /// Возвращает числовой идентификатор сущности для группировки версий.
    fn id(&self) -> i64;

    /// Вставляет новые версии для переданных элементов, не трогая существующие.
    ///
    /// Версия инкрементируется автоматически: берётся `max(pricing_version)`
    /// для каждого `id` из версионной таблицы и увеличивается на 1.
    /// Новые `id` получают версию 1.
    #[tracing::instrument(skip_all)]
    async fn insert_version_vec_returning(
        items: &[Self],
        pool: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<Vec<Self::Versioned>> {
        insert_vec_inner(items, pool).await
    }
}

/// Внутренняя реализация вставки версий.
///
/// Блокирует версионную таблицу (`ACCESS EXCLUSIVE`) перед чтением
/// максимальных версий, чтобы избежать гонки при параллельных вставках.
/// Блокировка снимается только при завершении всей транзакции -- поэтому
/// функцию нужно вызывать осторожно, чтобы не держать транзакцию слишком долго
/// и не спровоцировать deadlock.
async fn insert_vec_inner<T: DbVersioned>(
    items: &[T],
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<Vec<T::Versioned>> {
    let v_table = T::Versioned::TABLE;
    let lock_query = format!("LOCK TABLE {v_table} IN ACCESS EXCLUSIVE MODE");
    sqlx::query(&lock_query).execute(&mut *tx).await?;

    let ids = items.iter().map(|x| x.id()).collect::<Vec<_>>();
    let query_string = format!(
        "SELECT id,max(pricing_version) FROM {v_table} WHERE id=ANY($1) GROUP BY id",
    );
    tracing::info!(kind = "infra", "Input ids: {:?}", ids);

    // ver: id -> текущий максимальный номер версии.
    let mut ver = sqlx::query(&query_string)
        .bind(ids)
        .try_map(|r| <(i64, i16)>::from_row(&r))
        .fetch_all(&mut *tx)
        .await?
        .into_iter()
        .collect::<AHashMap<i64, i16>>();

    tracing::info!(kind = "infra", "Output (id, version_asez2)s: {:?}", ver);

    // Для каждого элемента: если id уже есть -- инкрементируем версию,
    // если нет -- начинаем с 1.
    let mut new_versions = items
        .iter()
        .map(|x| {
            let version = ver.entry(x.id()).and_modify(|x| *x += 1).or_insert(1);
            x.to_versioned(*version)
        })
        .collect::<Vec<_>>();
    T::Versioned::insert_vec_returning(&mut new_versions, &mut *tx).await
}
