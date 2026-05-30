use super::{
    db_item_core::DbItem, field_mask::DbAdaptorFieldMask, DbFieldMask, Field,
    Select,
};
use crate::result::Result;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use std::fmt::Debug;

/// Вспомогательный тип для однозначной сериализации опциональных полей адаптора.
///
/// Serde не умеет отличить `None` (поле отсутствует в JSON) от `null` без
/// дополнительных атрибутов. `DbAdaptorOption` используется с `#[serde(with = ...)]`,
/// чтобы поле сериализовалось без обёртки `{"Some": ...}`.
#[derive(Serialize, Deserialize)]
#[serde(remote = "Option")]
#[serde(untagged)]
pub enum DbAdaptorOption<T> {
    Some(T),
    None,
}

impl<T> Default for DbAdaptorOption<T> {
    fn default() -> Self {
        Self::None
    }
}

/// Трейт DTO-адаптора для [`DbItem`].
///
/// Каждое поле адаптора имеет тип `Option<T>`, что позволяет фронтенду
/// передавать только изменяемые поля. `None` означает "не трогать поле",
/// `Some(v)` -- обновить до `v`.
///
/// Реализуется через `#[derive(DbAdaptor)]`, которая по исходной структуре
/// `DbItem` генерирует теневую структуру `{Name}Rep` с `Option`-полями,
/// методы конвертации и вычисления масок.
#[async_trait::async_trait]
pub trait DbAdaptor:
    Debug + Sync + Send + for<'a> Deserialize<'a> + Serialize
{
    type DbItem: DbItem;
    /// Поля-дубликаты адаптора, которые не соответствуют колонкам БД.
    /// Нужны для дополнительных данных, которые дублируют существующие поля
    /// под другим именем (например, альтернативное имя для фронтенда).
    const DUP_FIELDS: &'static [&'static str];

    /// Конвертирует адаптор в [`DbItem`], разворачивая `Option`-поля.
    ///
    /// Если поле имеет `None`, оно заменяется дефолтным значением.
    fn into_item(self) -> Result<Self::DbItem>;

    /// Конвертирует [`DbItem`] в адаптор.
    ///
    /// Если `fields` не `None`, только перечисленные поля попадают в `Some(...)`,
    /// остальные остаются `None`. Первичные ключи включаются всегда.
    fn from_item<T>(item: Self::DbItem, fields: Option<&[T]>) -> Self
    where
        T: AsRef<str>,
    {
        let mask = fields.map_or_else(
            DbAdaptorFieldMask::all,
            DbAdaptorFieldMask::with_fields_and_pkeys,
        );
        Self::from_item_masked(item, &mask)
    }

    /// Конвертирует [`DbItem`] в адаптор с полным набором полей.
    fn from_item_full(item: Self::DbItem) -> Self {
        let mask = DbAdaptorFieldMask::all();
        Self::from_item_masked(item, &mask)
    }

    /// Конвертирует [`DbItem`] в адаптор по заданной маске.
    ///
    /// Поля с `mask=false` выставляются в `None`, остальные в `Some(...)`.
    fn from_item_masked(
        item: Self::DbItem,
        mask: &DbAdaptorFieldMask<Self>,
    ) -> Self;

    /// Возвращает маску полей, у которых в этом адапторе стоит `Some(...)`.
    ///
    /// Позволяет при массовом обновлении обновлять только переданные поля,
    /// а не все поля таблицы. Осторожно: если поле было `None` изначально,
    /// оно останется без изменений -- нет способа сбросить поле в дефолт
    /// через такую маску.
    fn bind_mask(&self) -> DbFieldMask<Self::DbItem>;

    /// Строит маску как объединение (`OR`) масок всех переданных адапторов.
    ///
    /// Поле входит в маску, если хотя бы в одном адапторе оно `Some(...)`.
    fn create_default_bind_mask<'a, I>(items: I) -> DbFieldMask<Self::DbItem>
    where
        I: IntoIterator<Item = &'a Self>,
        Self: 'a,
    {
        items.into_iter().fold(DbFieldMask::none(), |m, i| m | i.bind_mask())
    }

    /// Строгий вариант: поле входит в маску только если оно `Some(...)` у всех адапторов.
    ///
    /// Если у разных адапторов поле частично `Some`/`None` -- возвращает ошибку.
    fn create_strict_bind_mask(items: &[Self])
        -> Result<DbFieldMask<Self::DbItem>>;

    /// Мёрджит адаптор с существующим [`DbItem`]: поля `Some(...)` заменяют
    /// значения в `item`, поля `None` оставляют значения как есть.
    ///
    /// Полезно когда часть полей приходит из DTO, а часть нужно взять из БД.
    fn into_item_merged(self, item: Self::DbItem) -> Result<Self::DbItem>;

    /// Выставляет указанные поля в `Some(Default::default())`.
    ///
    /// Позволяет явно обнулить поля до значения по умолчанию, даже если
    /// они были `None`.
    fn zero_fields(self, field_mask: &DbFieldMask<Self::DbItem>) -> Self;

    /// Выставляет указанные поля в `None`.
    fn unset_fields(self, field_mask: &DbFieldMask<Self::DbItem>) -> Self;

    /// Мёрджит с [`DbItem`], но учитывает только поля из `mask`.
    ///
    /// Поля вне маски сбрасываются в `None` перед слиянием, т.е. не
    /// перезаписывают значения в `item`. Маска должна быть получена через
    /// [`make_bind_mask`](asez2_shared_db::db_item::make_bind_mask).
    fn into_item_merged_selected(
        self,
        item: Self::DbItem,
        mask: &DbFieldMask<Self::DbItem>,
    ) -> Result<Self::DbItem> {
        self.unset_fields(&mask.inverted()).into_item_merged(item)
    }

    /// Выбирает строки из таблицы и конвертирует их в адапторы.
    ///
    /// Маска полей адаптора строится из `q.field_list`, поэтому в ответе
    /// будут `Some(...)` только для запрошенных колонок.
    async fn select<'a, Ex, C>(q: &Select, pool: Ex) -> Result<C>
    where
        Ex: Executor<'a, Database = Postgres>,
        C: FromIterator<Self>,
    {
        let items = Self::DbItem::select(q, pool).await?;
        let mask = DbAdaptorFieldMask::with_fields_and_pkeys(&q.field_list);
        Ok(items.into_iter().adaptors_with_mask(mask).collect())
    }

    async fn select_maybe<'a, Ex>(q: &Select, pool: Ex) -> Result<Option<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        let item_maybe = Self::DbItem::select_option(q, pool).await?;
        Ok(item_maybe.map(|item| {
            let mask = DbAdaptorFieldMask::with_fields_and_pkeys(&q.field_list);
            Self::from_item_masked(item, &mask)
        }))
    }

    /// Вставляет срез [`DbItem`] и возвращает результат как адапторы (все поля).
    async fn insert_vec_returning<C: FromIterator<Self>>(
        items: &mut [Self::DbItem],
        pool: &mut sqlx::Transaction<Postgres>,
    ) -> Result<C> {
        let items = Self::DbItem::insert_vec_returning(items, pool).await?;
        Ok(items.into_iter().adaptors().collect())
    }

    /// Обновляет срез [`DbItem`] и возвращает результат как адапторы.
    ///
    /// Если `return_fields` равен `None`, структура должна иметь `#[sqlx(default)]`
    /// на всех полях -- иначе `FromRow` упадёт.
    async fn update_vec_returning<C: FromIterator<Self>>(
        items: &[Self::DbItem],
        update_fields: Option<&[&str]>,
        return_fields: Option<&[&str]>,
        pool: &mut sqlx::Transaction<Postgres>,
    ) -> Result<C> {
        let items = Self::DbItem::update_vec_returning(
            items,
            update_fields,
            return_fields,
            pool,
        )
        .await?;
        let mask = return_fields.map_or_else(
            DbAdaptorFieldMask::all,
            DbAdaptorFieldMask::with_fields_and_pkeys,
        );
        Ok(items.into_iter().adaptors_with_mask(mask).collect())
    }
}

/// Фабрика конвертера `DbItem -> DbAdaptor` для заданного списка полей.
///
/// Удобно при массовой конвертации данных за пределами итераторов над [`DbItem`]:
/// создаётся один раз, маска вычисляется один раз.
pub fn from_item_with_fields<'a, T, I, S>(fields: I) -> impl Fn(T::DbItem) -> T
where
    T: DbAdaptor,
    I: IntoIterator<Item = &'a S>,
    S: 'a + AsRef<str>,
{
    let mask = DbAdaptorFieldMask::with_fields_and_pkeys(fields);
    move |item| T::from_item_masked(item, &mask)
}

/// Трейт для получения всех полей адаптора вместе с их значениями.
///
/// Применяется при экспорте данных, когда нужно перебрать пары
/// `(имя_поля, значение)` без знания конкретного типа адаптора.
pub trait DbAdaptorFieldsWithValues: DbAdaptor {
    const FIELDS: &'static [&'static str];
    /// Возвращает все поля со значениями.
    fn fields_with_values(&self) -> Vec<Field>;

    /// Возвращает отображение `имя_поля -> индекс` для быстрого поиска.
    fn field_index_map<I>() -> I
    where
        I: FromIterator<(&'static str, usize)>,
    {
        Self::FIELDS
            .iter()
            .enumerate()
            .map(|(index, field_name)| (*field_name, index))
            .collect()
    }
}

/// Расширение для итераторов над [`DbItem`] -- конвертация в адапторы.
pub trait AdaptorableIter {
    /// Конвертирует каждый элемент в адаптор с полным набором полей.
    fn adaptors<A>(self) -> AdaptorIter<Self, A>
    where
        A: DbAdaptor,
        Self: Iterator<Item = A::DbItem> + Sized,
    {
        AdaptorIter {
            iter: self,
            mask: DbAdaptorFieldMask::all(),
        }
    }

    /// Конвертирует каждый элемент в адаптор с указанными полями.
    /// Первичные ключи всегда включаются в маску.
    fn adaptors_with_fields<'a, A, I, S>(self, fields: I) -> AdaptorIter<Self, A>
    where
        A: DbAdaptor,
        I: IntoIterator<Item = &'a S>,
        S: 'a + AsRef<str>,
        Self: Iterator<Item = A::DbItem> + Sized,
    {
        AdaptorIter {
            iter: self,
            mask: DbAdaptorFieldMask::with_fields_and_pkeys(fields),
        }
    }

    /// Конвертирует каждый элемент в адаптор по заданной маске.
    fn adaptors_with_mask<A>(
        self,
        mask: DbAdaptorFieldMask<A>,
    ) -> AdaptorIter<Self, A>
    where
        A: DbAdaptor,
        Self: Iterator<Item = A::DbItem> + Sized,
    {
        AdaptorIter { iter: self, mask }
    }
}

impl<I> AdaptorableIter for I {}

/// Итератор, конвертирующий [`DbItem`] в соответствующие адапторы.
///
/// Маска вычисляется один раз при создании итератора, а не для каждого элемента.
pub struct AdaptorIter<I, A>
where
    A: DbAdaptor,
    I: Iterator<Item = A::DbItem>,
{
    iter: I,
    mask: DbAdaptorFieldMask<A>,
}

impl<I, A> Iterator for AdaptorIter<I, A>
where
    A: DbAdaptor,
    I: Iterator<Item = A::DbItem>,
{
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| A::from_item_masked(item, &self.mask))
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use shared_db_derive::{DbAdaptor, DbItem};

    use super::*;
    use crate as asez2_shared_db;

    #[derive(Clone, Debug, Default, DbItem, DbAdaptor, Serialize, Deserialize)]
    #[adaptor_derive(Debug, Default, Serialize, Deserialize)]
    struct Itm {
        #[item_field_pkey]
        pub foo: i32,
    }

    fn items(n: usize) -> Vec<Itm> {
        (0..n).map(|i| Itm { foo: i as i32 }).collect()
    }

    #[test]
    fn adaptors() {
        let items = items(1);
        let _adaptors: Vec<ItmRep> = items.into_iter().adaptors().collect();
    }

    #[test]
    fn adaptors_with_mask() {
        let items = items(1);
        let _adaptors: Vec<_> = items
            .into_iter()
            .adaptors_with_mask(DbAdaptorFieldMask::<ItmRep>::all())
            .collect();
    }

    #[test]
    fn adaptors_with_fields() {
        let items = items(1);
        let _adaptors: Vec<ItmRep> =
            items.into_iter().adaptors_with_fields(&["a", "b", "foo"]).collect();
    }
}
