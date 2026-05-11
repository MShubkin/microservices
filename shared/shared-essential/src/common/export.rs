use std::{collections::HashMap, marker::PhantomData};

use crate::presentation::dto::{
    general::{DataRecord, DataRecords, FeWrapper, TaggedValue},
    UiValue,
};
use ahash::AHashMap;
use asez2_shared_db::{
    db_item::{AsezTimestamp, DbAdaptorFieldsWithValues},
    Value,
};
use sqlx::types::time::{PrimitiveDateTime, UtcOffset};

/// Используется для конструкции таблицы экспорта
/// Соотносит идентификатор поля с данными для одного элемента (т.е. 1 ряда)
pub trait FieldLookup<'fields> {
    type Source;
    type Field: 'fields;

    /// Создание структуры данных для поиск полям их значений
    fn build(input: Self::Source) -> Self;

    /// Ищет данные для соответстующего поля.
    /// Чаще всего поле это string like.
    fn get_or_null(&self, pos: usize, field: Self::Field) -> TaggedValue;
}

/// Соотносит название поля с данными из FeWrapper<T>, T - DbAdaptor.
/// Либо из Entity (должен быть #[adaptor_fields_with_values]), либо из extra_fields
pub struct FeWrapperRepLookup<T> {
    fields_tbl: AHashMap<&'static str, Option<Value>>,
    extra_fields: HashMap<String, UiValue>,

    // it's like a generic over FeWrapper<T>
    _phantom: PhantomData<T>,
}

impl<'fields, T> FieldLookup<'fields> for FeWrapperRepLookup<T>
where
    T: DbAdaptorFieldsWithValues,
{
    type Source = FeWrapper<T>;
    type Field = &'fields str;

    fn build(
        FeWrapper {
            entity,
            extra_fields,
            ..
        }: Self::Source,
    ) -> Self {
        Self {
            extra_fields,
            fields_tbl: entity
                .fields_with_values()
                .into_iter()
                .map(|item| (item.field(), item.value))
                .collect::<AHashMap<_, _>>(),
            _phantom: PhantomData,
        }
    }

    fn get_or_null(&self, _: usize, field: Self::Field) -> TaggedValue {
        if let Some(val) = self.fields_tbl.get(field).cloned() {
            val.into()
        } else if let Some(val) = self.extra_fields.get(field).cloned() {
            val.into()
        } else {
            TaggedValue::Null
        }
    }
}

/// Хелпер для конструирования таблицы для экспорта
/// См. [`crate::presentation::dto::general::DataRecords`]
pub fn build_export_table<'fields, L>(
    data: impl IntoIterator<Item = L::Source>,
    fields: impl IntoIterator<Item = L::Field> + Clone + 'fields,
) -> Vec<DataRecord>
where
    L: FieldLookup<'fields>,
{
    data.into_iter()
        .enumerate()
        .map(|(i, x)| {
            let lookup = L::build(x);

            fields
                .clone()
                .into_iter()
                .map(|field| lookup.get_or_null(i, field))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Конвертация времени в gmt+3
pub fn convert_timestamp_fields(data_records: &mut DataRecords) {
    data_records.data.iter_mut().flat_map(|item| item.iter_mut()).for_each(
        |item| {
            if let TaggedValue::DateTime(value) = item {
                let off_set_date_time =
                    value.0.assume_utc().to_offset(UtcOffset::hours(3));
                *value = AsezTimestamp(PrimitiveDateTime::new(
                    off_set_date_time.date(),
                    off_set_date_time.time(),
                ));
            }
        },
    );
}
