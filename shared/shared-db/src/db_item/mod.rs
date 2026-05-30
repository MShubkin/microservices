use crate::result::Result;

use ahash::AHashMap;
use sqlx::{Executor, Postgres};
use std::fmt::Debug;

pub mod date_time;
pub mod db_adaptor;
pub mod db_item_core;
pub mod db_item_del;
pub mod db_item_ext;
pub mod db_update_by_filter;
pub mod db_upsert;
pub mod field_mask;
pub mod int_array;
pub mod joined;
pub mod selection;
pub mod versioned;

pub mod db_enum_core;
pub use date_time::{AsezDate, AsezTimestamp};
pub use db_adaptor::*;
pub use db_enum_core::EnumDiscriminant;
pub use db_item_core::*;
pub use db_item_del::DbItemDel;
pub use db_item_ext::{DbItemExt, Field};
pub use db_update_by_filter::DbUpdateByFilter;
pub use db_upsert::DbUpsert;
pub use field_mask::{DbAdaptorFieldMask, DbFieldMask};
pub use selection::filters::{Filter, FilterTree};
pub use selection::{Select, SelectionKind};
pub use shared_db_derive::{DbAdaptor, DbItem, DbItemExt, DbUpsert};
pub use versioned::DbVersioned;
#[cfg(test)]
mod tests;

/// Преобразует массив полей в строку placeholder'ов для VALUES-выражения.
///
/// Аргумент `nth` указывает порядковый номер строки в массовой вставке,
/// чтобы счётчик `$N` не сбрасывался на единицу между строками.
/// Пример: `["id", "name", "address"]` при `nth=1` → `"($4,$5,$6)"`.
/// TODO: Перенести в proc-macro для устранения накладных расходов в рантайме.
pub fn field_counter(fields: &[&str], nth: usize) -> String {
    let mut output = String::with_capacity(fields.len() + 1);
    // SQL нумерует привязки с 1, поэтому offset начинается с 1.
    let offset = fields.len() * nth + 1;

    output.push('(');
    for i in offset..fields.len() + offset {
        output.extend(format!("${},", i).chars());
    }
    output.pop();
    output.push(')');

    output
}

/// Трейт для обратной совместимости полей между фронтендом и бэкендом.
///
/// Фронтенд иногда использует имена полей, отличающиеся от реальных имён
/// колонок в БД (например, переименованные поля после рефакторинга).
/// Трейт позволяет задать таблицу переименований [`TOLERATED`] и применить
/// её к [`Select`] перед выполнением запроса, не меняя API фронтенда.
pub trait FieldTolerance {
    /// Пары `(имя_фронтенда, имя_бэкенда)`.
    const TOLERATED: &'static [(&'static str, &'static str)] = &[];

    fn apply_tolerance_to_select(s: &mut Select) {
        for (front, back) in Self::TOLERATED {
            s.field_list.iter_mut().for_each(|f| map_field(f, front, back));
            s.order_list
                .iter_mut()
                .for_each(|f| map_field(&mut f.field, front, back));
            s.filter_list
                .slice_mut()
                .iter_mut()
                .for_each(|f| map_field(&mut f.field, front, back));
            s.distinct_on.iter_mut().for_each(|f| map_field(f, front, back));
        }
    }

    fn apply_tolerance_to_fields<'a>(fields: &'a [&'a str]) -> Vec<&'a str> {
        let mapping = Self::TOLERATED.iter().copied().collect::<AHashMap<_, _>>();
        let map_field = move |f| mapping.get(f).copied().unwrap_or(f);
        fields.iter().map(|x| map_field(x)).collect()
    }

    /// Переводит толерантное наименование в актуальное для сущности.
    ///
    /// Если было передано актуальное или в принципе невалидное поле для сущности,
    /// то оно и будет возвращено.
    fn actual_fieldname(fieldname: &str) -> &str {
        Self::TOLERATED
            .iter()
            .find(|(tolerated_name, _)| *tolerated_name == fieldname)
            .map(|(_, actual_name)| *actual_name)
            .unwrap_or(fieldname)
    }
}

/// Возвращает поля для UPDATE с учётом маски: включает первичные ключи,
/// исключает поля с `GENERATED ALWAYS AS` (autogen_always).
pub fn update_fields_helper<D: DbItem>(mask: &DbFieldMask<D>) -> Vec<&'static str> {
    mask.clone().with_pkeys().without_autogen().to_fields()
}

/// Маска полей для SET-части UPDATE: указанные поля минус autogen_always.
/// Первичные ключи в SET не входят -- они уходят в WHERE.
pub fn update_set_fields_helper<D: DbItem>(
    mask: &DbFieldMask<D>,
) -> Vec<&'static str> {
    mask.clone().without_autogen().to_fields()
}

/// Возвращает поля для SELECT: просто все включённые в маску поля.
pub fn select_fields_helper<D: DbItem>(mask: &DbFieldMask<D>) -> Vec<&'static str> {
    mask.to_fields()
}

/// Строит маску привязки по именам полей, автоматически включая первичные ключи.
///
/// Обратная операция к [`update_fields_helper`]: по именам полей
/// восстанавливает битовую маску, готовую к передаче в `bind_limited_fields`.
pub fn make_bind_mask<D: DbItem>(selected_fields: &[&str]) -> DbFieldMask<D> {
    DbFieldMask::make_bind_mask(selected_fields)
}

/// Строит маску привязки для одиночного UPDATE.
///
/// Отличается от [`make_bind_mask`] тем, что исключает autogen_always поля,
/// которые нельзя обновлять напрямую.
pub fn make_single_update_bind_mask<D: DbItem>(
    selected_fields: &[&str],
) -> DbFieldMask<D> {
    DbFieldMask::make_single_update_bind_mask(selected_fields)
}

fn map_field(f: &mut String, front: &str, back: &str) {
    if f as &str == front {
        *f = back.to_string();
    }
}
