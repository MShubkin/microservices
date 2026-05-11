use std::sync::Arc;

use ahash::AHashMap;
use asez2_shared_db::db_item::AsezDate;
use async_trait::async_trait;
use itertools::Itertools;
use shared_essential::presentation::dto::master_data::error::{
    MasterDataError, MasterDataResult,
};
use tokio::sync::{OnceCell, RwLock};

use crate::presentation::dto::{
    DicrionaryHierarchyResData, DicrionaryHierarchyResItem,
};

use super::master_data::get_master_data;

/// Общий кеш для поиска по многоуровневым справочникам.
static HIERARCHICAL_VALUES: OnceCell<
    RwLock<MasterDataResult<Arc<HierarchicalValues>>>,
> = OnceCell::const_new();

pub(crate) async fn get_hierarchical_values(
) -> MasterDataResult<Arc<HierarchicalValues>> {
    let g = HIERARCHICAL_VALUES
        .get_or_init(|| async {
            RwLock::new(build_hierarchical_values().await.map(Arc::new))
        })
        .await
        .read()
        .await;
    (*g).clone()
}

async fn build_hierarchical_values() -> MasterDataResult<HierarchicalValues> {
    let master_data = get_master_data()?;

    let mut map = AHashMap::new();

    let mut insert_hierarchy = |(name, hierarchy): (&'static str, Hierarchy)| {
        map.insert(name, hierarchy);
    };

    master_data.okpd2.generate().await.map(&mut insert_hierarchy)?;
    master_data.category.generate().await.map(&mut insert_hierarchy)?;
    master_data.budget_item.generate().await.map(&mut insert_hierarchy)?;

    Ok(HierarchicalValues(map))
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum HierarchicalValuesError<'a> {
    #[error("многоуровневый справочник с именем {0} не поддерживается")]
    NoSuchTree(&'a str),
    #[error("значение {0} отсутствует в многоуровневом справочнике {1}")]
    NoSuchItem(i32, &'a str),
}

impl<'a> From<HierarchicalValuesError<'a>> for MasterDataError {
    fn from(error: HierarchicalValuesError<'a>) -> Self {
        MasterDataError::InternalError(error.to_string())
    }
}

/// Реестр иерархий по многоуровневым справочникам.
///
/// Отображение имени справочника в иерархию идентификаторов этого справочника.
#[derive(Debug)]
pub(crate) struct HierarchicalValues(AHashMap<&'static str, Hierarchy>);

impl HierarchicalValues {
    pub(crate) fn is_in_subtree<'a>(
        &'a self,
        name: &'a str,
        root: i32,
        value: i32,
    ) -> Result<bool, HierarchicalValuesError<'a>> {
        let hierarchy = self.get(name)?;
        hierarchy
            .contains(root, value, AsezDate::today())
            .ok_or(HierarchicalValuesError::NoSuchItem(value, name))
    }

    pub(crate) fn get_ui_hierarchy<'a>(
        &'a self,
        name: &'a str,
    ) -> Result<Vec<DicrionaryHierarchyResItem>, HierarchicalValuesError<'a>> {
        let hierarchy = self.get(name)?;
        Ok(hierarchy.get_ui_hierarchy(AsezDate::today()))
    }

    fn get<'a>(
        &'a self,
        name: &'a str,
    ) -> Result<&'a Hierarchy, HierarchicalValuesError<'a>> {
        self.0.get(&name).ok_or(HierarchicalValuesError::NoSuchTree(name))
    }
}

impl FromIterator<(&'static str, Hierarchy)> for HierarchicalValues {
    fn from_iter<T: IntoIterator<Item = (&'static str, Hierarchy)>>(
        iter: T,
    ) -> Self {
        HierarchicalValues(iter.into_iter().collect())
    }
}

#[derive(Debug)]
pub(crate) struct HierarchyData {
    pub parents: Vec<i32>,
    pub code: String,
    pub is_removed: bool,
    pub from_date: AsezDate,
    pub to_date: AsezDate,
}

impl HierarchyData {
    fn is_valid(&self, date: &AsezDate) -> bool {
        let HierarchyData {
            is_removed,
            from_date,
            to_date,
            ..
        } = self;
        !is_removed && (from_date..=to_date).contains(&date)
    }
}

/// Иерархия идентификаторов многоуровневого справочника.а
///
/// Для каждого идентификатора хранится список его "родительских" идентификаторов и текст (код).
#[derive(Debug, Default)]
pub(crate) struct Hierarchy(AHashMap<i32, HierarchyData>);

impl Hierarchy {
    fn contains(&self, root: i32, value: i32, date: AsezDate) -> Option<bool> {
        self.0
            .get(&value)
            .map(|data| data.is_valid(&date) && data.parents.contains(&root))
    }

    fn get_ui_hierarchy(&self, date: AsezDate) -> Vec<DicrionaryHierarchyResItem> {
        self.0
            .iter()
            .filter(|(_, data)| data.is_valid(&date))
            .map(|(id, HierarchyData { parents, code, .. })| {
                DicrionaryHierarchyResItem {
                    id: *id,
                    code: code.clone(),
                    parent_id: parents.last().copied().unwrap_or(0),
                }
            })
            .sorted_by(|a, b| a.code.cmp(&b.code))
            .collect()
    }
}

impl FromIterator<(i32, HierarchyData)> for Hierarchy {
    fn from_iter<T: IntoIterator<Item = (i32, HierarchyData)>>(iter: T) -> Self {
        Hierarchy(iter.into_iter().collect())
    }
}

impl From<AHashMap<i32, HierarchyData>> for Hierarchy {
    fn from(value: AHashMap<i32, HierarchyData>) -> Self {
        Hierarchy(value)
    }
}

#[async_trait]
pub(crate) trait HierarchyGenerator {
    async fn generate(&self) -> MasterDataResult<(&'static str, Hierarchy)>;
}

/// Трейт для потенциально иерархических данных
#[allow(dead_code)]
pub(crate) trait HierarchyEntry {
    type Id;

    /// Идентификатор записи
    fn id(&self) -> Self::Id;

    /// Идентфиикатор родительской записи
    fn parent_id(&self) -> Self::Id;

    /// Код записи
    fn code(&self) -> &str;

    /// Признак удаления записи.
    fn is_removed(&self) -> bool;

    /// Дата начала действия записи.
    fn get_from_date(&self) -> AsezDate;

    /// Дата окончания действия записи.
    fn get_to_date(&self) -> AsezDate;

    /// Является ли родительским элементом
    fn is_parent_of<T>(&self, other: &T) -> bool
    where
        T: HierarchyEntry,
        Self::Id: PartialEq<T::Id>,
    {
        self.id() == other.parent_id()
    }
}

/// Реализация `/v1/get/hierarchy/{dictionary}`.
pub(crate) async fn get_hierarchy(
    dictionary: &str,
) -> MasterDataResult<DicrionaryHierarchyResData> {
    let values = get_hierarchical_values().await?;
    let dictionary_list = values.get_ui_hierarchy(dictionary)?;

    Ok(DicrionaryHierarchyResData { dictionary_list })
}

#[cfg(test)]
mod tests {
    use asez2_shared_db::asez_date;

    use super::*;

    macro_rules! test_item {
        ($id: expr, $parents: expr, $is_removed:expr, $from:expr, $to:expr) => {
            (
                $id,
                HierarchyData {
                    parents: $parents.to_vec(),
                    code: Default::default(),
                    is_removed: $is_removed,
                    from_date: asez_date!($from),
                    to_date: asez_date!($to),
                },
            )
        };
    }

    #[test]
    fn validity() {
        let date = asez_date!("2025-08-12");
        #[rustfmt::skip]
        let h: Hierarchy = [
            test_item!(0,  [],                   false, "2020-01-01", "2029-12-31"),
            test_item!(1,  [0],                  false, "2020-01-01", "2029-12-31"),
            test_item!(2,  [0],                  false, "2020-01-01", "2029-12-31"),
            test_item!(3,  [0, 1],               true,  "2020-01-01", "2029-12-31"),
            test_item!(4,  [0, 1],               false, "2030-01-01", "2039-12-31"),
            test_item!(5,  [0, 2],               false, "2020-01-01", "2029-12-31"),
            test_item!(6,  [0, 2],               false, "2020-01-01", "2029-12-31"),
            test_item!(7,  [0, 1, 3],            true,  "2020-01-01", "2029-12-31"),
            test_item!(8,  [0, 1, 3],            true,  "2020-01-01", "2029-12-31"),
            test_item!(9,  [0, 1, 4],            false, "2030-01-01", "2039-12-31"),
            test_item!(10, [0, 1, 4],            false, "2030-01-01", "2039-12-31"),
            test_item!(11, [0, 1, 2, 4, 6, 10],  false, "2020-01-01", "2029-12-31"),
        ].into_iter().collect();

        assert_eq!(h.contains(2, 5, date), Some(true));
        assert_eq!(h.contains(3, 7, date), Some(false));
        assert_eq!(h.contains(4, 9, date), Some(false));
    }
}
