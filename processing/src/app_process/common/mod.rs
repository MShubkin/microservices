use crate::common::Result;

use asez2_shared_db::{
    db_item::{update_fields_helper, DbItemExt},
    DbAdaptor,
};
use shared_essential::application::records::{ProcessUpsert, Recorder};
use shared_essential::presentation::dto::response_request::Messages;

pub mod agenda;
pub mod plan;
pub mod protocol;

/// TODO: Обобщить для разных типов.
/// Функция выискивает записи которые там уже есть, их обновляет, а остальные
/// вставляет как новые.
pub(crate) async fn upsert<T: ProcessUpsert>(
    items: Vec<T>,
    messages: &mut Messages,
    fields: &[&'static str],
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    recorder.process_upsert(items, fields, messages).await?;
    Ok(())
}

/// Добавляем "plan_id" к списку полей для последующего `PlanRep::from_item(x, Some(RETURN_FIELDS)`
/// TODO: Можно еще больше обобщить, добавив список добавляемых полей как аргумент
#[allow(dead_code)]
pub(crate) fn add_plan_id_to_fields<'a, T: DbItemExt>(item: &T) -> Vec<&'a str> {
    let mut fields = item
        .fields_with_values()
        .iter()
        .filter(|x| x.value.is_some())
        .map(|x| x.field())
        .collect::<Vec<_>>();
    fields.push("plan_id");
    fields
}

#[derive(Debug)]
pub(crate) struct ItemsWithFields<T: ProcessUpsert> {
    pub(crate) items: Vec<T>,
    pub(crate) fields: Vec<&'static str>,
}

impl<T: ProcessUpsert> ItemsWithFields<T> {
    pub(crate) fn new<S>(database_items: Vec<S>) -> Result<Self>
    where
        S: DbAdaptor<DbItem = T>,
        Vec<T>: FromIterator<<S as DbAdaptor>::DbItem>,
    {
        // НБ. Надо раньше или позже учесть возможноьт разных полей, но пока
        // единственное такое поле это uuid, и можно на уровне по выше проработать.
        let field_mask = S::create_default_bind_mask(&database_items);
        let fields = update_fields_helper::<T>(&field_mask);
        let items = database_items
            .into_iter()
            .map(|database_item| database_item.into_item().map_err(Into::into))
            .collect::<Result<_>>()?;

        Ok(Self { items, fields })
    }

    /// Конструирование экземпляра напрямую из элементов БД и полей, требующих изменений.
    pub(crate) fn from_items_fields<S: Into<T>, I1: IntoIterator<Item = S>>(
        items: I1,
        fields: &[&'static str],
    ) -> Self {
        let items = items.into_iter().map(S::into).collect();
        let fields = fields.to_vec();
        ItemsWithFields { items, fields }
    }

    pub(crate) async fn update_all(
        self,
        messages: &mut Messages,
        recorder: &mut Recorder<'_>,
    ) -> Result<Vec<T>> {
        // НБ. Надо раньше или позже учесть возможноьт разных полей
        let ItemsWithFields { items, fields } = self;
        Ok(recorder.process_update(items, &fields, messages).await?)
    }

    pub(crate) async fn upsert_all(
        self,
        messages: &mut Messages,
        recorder: &mut Recorder<'_>,
    ) -> Result<Vec<T>> {
        // НБ. Надо раньше или позже учесть возможноьт разных полей
        let ItemsWithFields { items, fields } = self;

        recorder
            .process_upsert(items, &fields, messages)
            .await
            .map_err(Into::into)
    }
}
