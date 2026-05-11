use super::{
    db_item_core::DbItem, field_mask::DbAdaptorFieldMask, DbFieldMask, Field,
    Select,
};
use crate::result::Result;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use std::fmt::Debug;

/// This exists for unambiguous serialization of DbAdaptor.
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

/// This trait exists as a defined adaptor so that we can deserialize `DbItem` from Data Transfer Objects.
/// In other words, all D.T.Os that we use as stand-ins for database objects must implement this.
#[async_trait::async_trait]
pub trait DbAdaptor:
    Debug + Sync + Send + for<'a> Deserialize<'a> + Serialize
{
    type DbItem: DbItem;
    const DUP_FIELDS: &'static [&'static str];

    fn into_item(self) -> Result<Self::DbItem>;

    /// An optional list of fields to send is supplied. This allows us to deactivate
    /// certain fields before conversion and send only the remaining fields.
    /// NB: We do not apply tolerance internally here.
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

    /// Returns adapter with full set of fields.
    fn from_item_full(item: Self::DbItem) -> Self {
        let mask = DbAdaptorFieldMask::all();
        Self::from_item_masked(item, &mask)
    }

    fn from_item_masked(
        item: Self::DbItem,
        mask: &DbAdaptorFieldMask<Self>,
    ) -> Self;

    /// Returns mask of fields set to `Some()` in this adaptor.
    ///
    /// This is not perfect, but still allows us to reduce the number of fields we
    /// update. This bind mask should be used with care as it can lead to values
    /// never being updated back to default.
    fn bind_mask(&self) -> DbFieldMask<Self::DbItem>;

    /// Returns mask of fields set to `Some()` in any of the adaptors `items`.
    ///
    /// This is not perfect, but still allows us to reduce the number of fields we
    /// update. This bind mask should be used with care as it can lead to values
    /// never being updated back to default.
    fn create_default_bind_mask<'a, I>(items: I) -> DbFieldMask<Self::DbItem>
    where
        I: IntoIterator<Item = &'a Self>,
        Self: 'a,
    {
        items.into_iter().fold(DbFieldMask::none(), |m, i| m | i.bind_mask())
    }

    /// This is a stricter version of `create_default_bind_mask` which needs all items
    /// to have `Some` in the field before for it to be marked as `true`. If items
    /// have mixed Some/None values for a field, an error is returned.
    fn create_strict_bind_mask(items: &[Self])
        -> Result<DbFieldMask<Self::DbItem>>;

    /// Merges `item` with exising (non-`None``) values from this adaptor.
    ///
    /// This is useful when some business logic needs some fields received from a DTO and others from DB.
    fn into_item_merged(self, item: Self::DbItem) -> Result<Self::DbItem>;

    /// This function sets the listed fields to `Some(Default::default())`.
    ///
    /// Even if the field is set as `None` initially, it will subsequently be set to `Some(Default::default())`
    fn zero_fields(self, field_mask: &DbFieldMask<Self::DbItem>) -> Self;

    /// This function sets the listed fields to `None`.
    fn unset_fields(self, field_mask: &DbFieldMask<Self::DbItem>) -> Self;

    /// Merges `item` with exising (non-`None``) values from this adaptor selected by the `mask`.
    ///
    /// Note that the `mask` parameter should be obtained by the
    /// [`make_bind_mask`](asez2_shared_db::db_item::make_bind_mask) function.
    ///
    /// This is useful when some business logic needs some fields received from
    /// a DTO and others from DB, and DTO can contain some fields that are not
    /// relevant.
    fn into_item_merged_selected(
        self,
        item: Self::DbItem,
        mask: &DbFieldMask<Self::DbItem>,
    ) -> Result<Self::DbItem> {
        self.unset_fields(&mask.inverted()).into_item_merged(item)
    }

    /// This selects items from the table based on criteria and using the `Select` structure. Simple filtration
    /// operations as well as ordering and selection of specific fields is also possible.
    /// Trying to specify fields in the wrong order may cause an error to be returned, since the return structure
    /// is not flexible in the order of fields.
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

    /// Inserts a vec of items into the table returning the inserted rows. (See [`DbItem::insert_vec`])
    async fn insert_vec_returning<C: FromIterator<Self>>(
        items: &mut [Self::DbItem],
        pool: &mut sqlx::Transaction<Postgres>,
    ) -> Result<C> {
        let items = Self::DbItem::insert_vec_returning(items, pool).await?;
        Ok(items.into_iter().adaptors().collect())
    }

    /// This works as `update_vec`, however, it additionally takes a list of fields
    /// to return. And returns updated objects.
    ///
    /// NB: If the item does NOT have the `#[sqlx(default)]` attribute then
    /// `return_fields` must always be set to `None`.
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

/// Returns a function that converts an item into corresponding adaptor.
///
/// Useful when mass conversion is needed for data that is not an item iterator.
pub fn from_item_with_fields<'a, T, I, S>(fields: I) -> impl Fn(T::DbItem) -> T
where
    T: DbAdaptor,
    I: IntoIterator<Item = &'a S>,
    S: 'a + AsRef<str>,
{
    let mask = DbAdaptorFieldMask::with_fields_and_pkeys(fields);
    move |item| T::from_item_masked(item, &mask)
}

/// This trait is designed to get all the fields of the DbAdaptor entity along with their values
/// In particular, it is used for export functions
pub trait DbAdaptorFieldsWithValues: DbAdaptor {
    const FIELDS: &'static [&'static str];
    /// Get all fields with values
    fn fields_with_values(&self) -> Vec<Field>;

    /// Returns mapping from field name to index.
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

/// Trait extending `Iterator` of db items with operations that convert elements
/// to corresponding adaptors.
pub trait AdaptorableIter {
    /// Convert each item into adaptor with all fields.
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

    /// Convert each item into adaptor with specified fields (private key fields are always included).
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

    /// Convert each item into adaptor using specified field mask.
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

/// Iterator converting db items into their adaptors.
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
