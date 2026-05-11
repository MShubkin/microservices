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

/// Converts the field array into a representation of the bindings.
///
/// The `nth` argument tells us which item this is in the list that the counter may
/// be adjusted correctly.
/// eg `["id", "name", "address"] becomes `"($1,$2,$3)"`.
/// TODO: Convert into a proc macro.
pub fn field_counter(fields: &[&str], nth: usize) -> String {
    let mut output = String::with_capacity(fields.len() + 1);
    // Remember, SQl queries index starting at 1, eg ($1,$2).
    let offset = fields.len() * nth + 1;

    output.push('(');
    for i in offset..fields.len() + offset {
        output.extend(format!("${},", i).chars());
    }
    output.pop();
    output.push(')');

    output
}

/// A trait that allows checking and renaming of fields in select queries.
/// It is used to tolerate certain fields with "incorrect" or "front end correct"
/// names.
///
/// The only thing that the user needs to do to implement this trait is to write out
/// the mapping constant.
pub trait FieldTolerance {
    /// Frontend field is first, backend field is second.
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

    /// Переводит толерантное наименование в актуальное для сущности
    ///
    /// Если было передано актуальное или в принципе невалидное поле для сущности,
    /// то оно и будет возвращено
    fn actual_fieldname(fieldname: &str) -> &str {
        Self::TOLERATED
            .iter()
            .find(|(tolerated_name, _)| *tolerated_name == fieldname)
            .map(|(_, actual_name)| *actual_name)
            .unwrap_or(fieldname)
    }
}

pub fn update_fields_helper<D: DbItem>(mask: &DbFieldMask<D>) -> Vec<&'static str> {
    mask.clone().with_pkeys().without_autogen().to_fields()
}

/// Маска полей для update -- указанные поля минус autogen_always.
pub fn update_set_fields_helper<D: DbItem>(
    mask: &DbFieldMask<D>,
) -> Vec<&'static str> {
    mask.clone().without_autogen().to_fields()
}

pub fn select_fields_helper<D: DbItem>(mask: &DbFieldMask<D>) -> Vec<&'static str> {
    mask.to_fields()
}

/// Reverse of update_fields_helper.
pub fn make_bind_mask<D: DbItem>(selected_fields: &[&str]) -> DbFieldMask<D> {
    DbFieldMask::make_bind_mask(selected_fields)
}

/// Reverse of update_fields_helper.
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
