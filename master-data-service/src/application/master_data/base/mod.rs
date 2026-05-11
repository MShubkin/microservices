pub mod budget_item;
pub mod category;
pub mod okpd2;
pub mod plan_reasons_cancel;

use ahash::AHashMap;
use async_trait::async_trait;
use itertools::Itertools;
use sqlx::PgPool;
use std::collections::BTreeMap;

use asez2_shared_db::db_item::Select;
use asez2_shared_db::DbItem;
use tokio::sync::RwLock;

use asez2_shared_db::db_item::AsezTimestamp;
use shared_essential::{
    domain::master_data::*,
    presentation::dto::{
        master_data::{
            error::{MasterDataError, MasterDataResult},
            request::SearchByUserInput,
        },
        response_request::{MessageKind, Messages},
    },
};

use crate::{
    application::hierarchical_values::{HierarchyData, HierarchyEntry},
    domain::{MasterDataDirectoryInterface, MasterDataRecord},
};

macro_rules! master_data_record {
    ($record:ident, $($field:ident),* $(,)?) => {
        impl MasterDataRecord for $record {
            fn id(&self) -> i32 {
                self.id as i32
            }

            fn changed_at(&self) -> AsezTimestamp {
                self.changed_at
            }

            fn search_record(&self, search: &str) -> bool {
                $(
                    contains_case_insensitive(&self.$field, search)
                )||*
            }
        }
    };
}

master_data_record!(AssigningExecutorMethod, name);
master_data_record!(AttachmentType, name);
master_data_record!(BudgetItem, code, text);
master_data_record!(Category, code, text);
master_data_record!(Okpd2, code, text);
master_data_record!(EstimatedCommissionAgendaStatus, name);
master_data_record!(EstimatedCommissionProtocolStatus, name);
master_data_record!(EstimatedCommissionProtocolType, name);
master_data_record!(EstimatedCommissionResult, name);
master_data_record!(EstimatedCommissionRole, name);
master_data_record!(ExpertConclusionType, name);
master_data_record!(ObjectType, name);
master_data_record!(Organization, text, text_full, inn, kpp);
master_data_record!(OrganizationalStructure, text, text_short);
master_data_record!(OutputForm, text);
master_data_record!(PaymentCondition, name);
master_data_record!(PlanReasonCancelHeader, text);
master_data_record!(PlanReasonCancelImpactArea, text);
master_data_record!(PlanReasonCancelFunctionality, text);
master_data_record!(PlanReasonCancelCheckReason, text);
master_data_record!(PpzType, name);
master_data_record!(AnalysisMethod, name);
master_data_record!(PriceAnalysisMethod, name);
master_data_record!(PricingUnit, name);
master_data_record!(Response, text);
master_data_record!(SchedulerRequestUpdateCatalog, event_name);
master_data_record!(PriceInformationRequestType, name);
master_data_record!(TcpStatus, name);

impl MasterDataRecord for CriticalTypeColorScheme {
    fn id(&self) -> i32 {
        self.id as i32
    }
    fn changed_at(&self) -> AsezTimestamp {
        self.changed_at
    }
    fn search_record(&self, search: &str) -> bool {
        contains_case_insensitive(&self.name, search)
            || contains_case_insensitive(&self.color_code.to_string(), search)
    }
}

impl MasterDataRecord
    for shared_essential::domain::master_data::FavoriteDictionary
{
    fn id(&self) -> i32 {
        self.id
    }
    fn changed_at(&self) -> AsezTimestamp {
        AsezTimestamp::default()
    }
    fn search_record(&self, search: &str) -> bool {
        contains_case_insensitive(&self.name, search)
            || contains_case_insensitive(&self.text, search)
    }
}

/// Типовой справочник
#[derive(Debug, Default)]
pub struct MasterDataCommonDirectory<T: MasterDataRecord> {
    pub(crate) changed_at: AsezTimestamp,
    data: RwLock<BTreeMap<i32, T>>,
}

#[async_trait]
impl<T> MasterDataDirectoryInterface<T> for MasterDataCommonDirectory<T>
where
    T: MasterDataRecord + DbItem,
{
    async fn load(&mut self, pool: &PgPool) -> MasterDataResult<()> {
        let records = T::select(&Select::with_fields::<&str, _>([]), pool).await?;
        self.init_data(records).await?;
        Ok(())
    }

    async fn get_by_ids(
        &self,
        ids: &[i32],
    ) -> MasterDataResult<(Messages, Vec<T>)> {
        let map = self.data.read().await;
        let mut messages = Messages::default();
        let list = ids
            .iter()
            .filter_map(|id| {
                if let Some(record) = map.get(id) {
                    Some(record.clone())
                } else {
                    messages.add_message(
                        MessageKind::Error,
                        format!("Запись с id: {id} не найдена"),
                    );
                    None
                }
            })
            .collect();
        Ok((messages, list))
    }

    async fn get_by_id(&self, id: &i32) -> MasterDataResult<T> {
        let map = self.data.read().await;
        if let Some(record) = map.get(id) {
            Ok(record.clone())
        } else {
            Err(MasterDataError::InternalError(format!(
                "Запись с id: {id} не найдена",
            )))
        }
    }

    async fn search(
        &self,
        search_request: &SearchByUserInput,
    ) -> MasterDataResult<(Messages, Vec<T>)> {
        let SearchByUserInput {
            from,
            search,
            quantity,
        } = search_request;
        let map = self.data.read().await;
        let search = search.trim().to_lowercase();
        let result = map
            .values()
            .filter(|record| record.search_record(&search))
            .skip(*from as usize)
            .take(*quantity as usize)
            .cloned()
            .collect();
        Ok((Default::default(), result))
    }

    async fn get_updates(
        &self,
        timestamp: AsezTimestamp,
    ) -> MasterDataResult<Vec<T>> {
        let map = self.data.read().await;
        let is_zero_time = timestamp.unix_timestamp() == 0;
        let result_vec = map
            .iter()
            .filter_map(|(_, value)| {
                if is_zero_time || value.changed_at() > timestamp {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(result_vec)
    }

    async fn get_full_data(&self) -> MasterDataResult<Vec<T>> {
        let map = self.data.read().await;
        let result = map.values().cloned().collect();
        Ok(result)
    }
}

impl<T: MasterDataRecord> MasterDataCommonDirectory<T> {
    // const THRESHOLD: usize = 10;

    async fn init_data(&mut self, list: Vec<T>) -> MasterDataResult<()> {
        let mut map = self.data.write().await;
        map.clear();
        // TODO: Enable when HashMap
        // if map.capacity() < list.len() {
        //     map.reserve(list.len() + Self::THRESHOLD);
        // }

        map.extend(list.into_iter().map(|value| {
            let changed_at = value.changed_at();
            if self.changed_at < changed_at {
                self.changed_at = changed_at;
            }
            (value.id(), value)
        }));

        Ok(())
    }
}

/// Создание базовой иерархии по `id` и `parent_id` записей
fn build_common_hierarchy<T: HierarchyEntry>(
    items: Vec<T>,
) -> AHashMap<i32, HierarchyData>
where
    T::Id: Into<i32>,
{
    let mut first_hierarchy = AHashMap::with_capacity(items.len());
    let mut hierarchy = AHashMap::with_capacity(items.len());

    // Сначала собираем всех родителей на один уровень выше
    for item in items {
        let id = item.id().into();
        let parent_id = item.parent_id().into();

        first_hierarchy
            .entry(id)
            .or_insert_with(|| HierarchyData {
                parents: Vec::with_capacity(1),
                code: item.code().to_owned(),
                is_removed: item.is_removed(),
                from_date: item.get_from_date(),
                to_date: item.get_to_date(),
            })
            .parents
            .push(parent_id);
    }

    // Затем строим полноценную иерархию
    for (
        &id,
        HierarchyData {
            parents,
            code,
            is_removed,
            from_date,
            to_date,
        },
    ) in &first_hierarchy
    {
        let mut all_parents = parents.clone();
        let mut stack = all_parents.clone();

        while let Some(parent) = stack.pop() {
            if let Some(grandparents) = first_hierarchy.get(&parent) {
                for &higher_parent in &grandparents.parents {
                    if !all_parents.contains(&higher_parent) {
                        all_parents.push(higher_parent);
                        stack.push(higher_parent);
                    }
                }
            }
        }

        hierarchy.insert(
            id,
            HierarchyData {
                code: code.to_owned(),
                parents: all_parents.into_iter().sorted().collect(),
                is_removed: *is_removed,
                from_date: *from_date,
                to_date: *to_date,
            },
        );
    }

    hierarchy
}

fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    let (haystack_len, needle_len) =
        (haystack.chars().count(), needle.chars().count());

    if needle_len > haystack_len {
        return false;
    }

    for idx in 0..=(haystack_len - needle_len) {
        if haystack
            .chars()
            .flat_map(char::to_lowercase)
            .skip(idx)
            .zip(needle.chars())
            .all(|(haystack_c, needle_c)| haystack_c == needle_c)
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod build_common_hierarchy {
    use ahash::AHashMap;
    use asez2_shared_db::db_item::AsezDate;

    use super::build_common_hierarchy;
    use crate::application::hierarchical_values::HierarchyEntry;

    macro_rules! test_item {
        ($id: expr, $parent_id: expr) => {
            TestItem {
                id: $id,
                parent_id: $parent_id,
            }
        };
    }

    #[derive(Default)]
    struct TestItem {
        id: i32,
        parent_id: i32,
    }

    impl HierarchyEntry for TestItem {
        type Id = i32;

        fn id(&self) -> Self::Id {
            self.id
        }

        fn parent_id(&self) -> Self::Id {
            self.parent_id
        }

        fn code(&self) -> &str {
            ""
        }

        fn is_removed(&self) -> bool {
            false
        }

        fn get_from_date(&self) -> AsezDate {
            Default::default()
        }

        fn get_to_date(&self) -> AsezDate {
            Default::default()
        }
    }

    #[test]
    fn basic() {
        let mut hierarchy = build_common_hierarchy(vec![
            test_item!(0, 0),
            test_item!(1, 0),
            test_item!(2, 0),
            test_item!(3, 1),
            test_item!(4, 1),
            test_item!(5, 2),
            test_item!(6, 2),
            test_item!(7, 3),
            test_item!(8, 3),
            test_item!(9, 4),
            test_item!(10, 4),
            test_item!(11, 10),
            test_item!(11, 6),
        ]);

        let expected: AHashMap<i32, Vec<i32>> = AHashMap::from_iter([
            (0, vec![0]),
            (1, vec![0]),
            (2, vec![0]),
            (3, vec![0, 1]),
            (4, vec![0, 1]),
            (5, vec![0, 2]),
            (6, vec![0, 2]),
            (7, vec![0, 1, 3]),
            (8, vec![0, 1, 3]),
            (9, vec![0, 1, 4]),
            (10, vec![0, 1, 4]),
            (11, vec![0, 1, 2, 4, 6, 10]),
        ]);

        expected.into_iter().for_each(|(id, expected)| {
            let parents = hierarchy.remove(&id).unwrap().parents;
            assert_eq!(
                parents, expected,
                "Ожидаемые родители {:?} для {} не равны фактическим {:?}",
                expected, id, parents
            )
        })
    }
}

#[cfg(test)]
mod contains_case_insensetive {
    use crate::application::master_data::base::contains_case_insensitive;

    #[test]
    fn haystack_needle_equal_len() {
        let haystack = "some_search";
        let wrong_needle = "Some_searc)";

        assert!(!contains_case_insensitive(haystack, wrong_needle));
        assert!(contains_case_insensitive(haystack, haystack));
    }

    #[test]
    fn needle_bigger_len() {
        let haystack = "some_search";
        let wrong_needle = "Some_search_";

        assert!(!contains_case_insensitive(haystack, wrong_needle));
    }

    #[test]
    fn haystack_bigger_len() {
        let haystack = "какой то поиск";
        let wrong_needle = "писк";
        let success_needle1 = " то ";
        let success_needle2 = "поиск";

        assert!(!contains_case_insensitive(haystack, wrong_needle));
        assert!(contains_case_insensitive(haystack, success_needle1));
        assert!(contains_case_insensitive(haystack, success_needle2));
    }

    #[test]
    fn photi() {
        let haystack =
            "Рыба и прочая продукция рыболовства; услуги, связанные с рыболовством";
        let needle = "рыба";

        assert!(contains_case_insensitive(haystack, needle));
    }
}
