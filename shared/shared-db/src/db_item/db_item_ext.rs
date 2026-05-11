//! This module contains the `DbItemExt` trait which
//! contains functionality which is not essential for `DbItem`
//! to function but is useful for certain additional functionality
//! such as checking rules and comparing field changes in historian.
use super::*;
use crate::Value;

use uuid::Uuid;

/// Describes a field of an item, much like a DbField. It contains
/// the name of a field, which is static from creation and its value.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    field: &'static str,
    pub value: Option<Value>,
}

impl Field {
    pub fn field(&self) -> &'static str {
        self.field
    }

    pub fn new<V: Into<Value>>(field: &'static str, value: Option<V>) -> Self {
        Self {
            field,
            value: value.map(Into::into),
        }
    }
}

/// This is a helper trait that allows interface between `DbItem` and
/// functionality in `processing`. It should be implemented via proc_macro.
/// Even if the structure itself does not implement `PartialEq` or `Clone`, for
/// the derivation to work all of its fields should.
#[async_trait::async_trait]
pub trait DbItemExt: DbItem {
    /// The uuid of the record (DbItem).
    /// It should be noted that for now this functionality will only
    /// work properly for items that use uuid as the primary key as the historian
    /// table uses `record_uuid` as the definitive primary key of the item
    fn record_uuid(&self) -> Uuid;
    // Get pkeys and their values.
    fn pkeys_with_values(&self) -> Vec<Field>;

    fn fields_with_values(&self) -> Vec<Field>;

    /// Should return all fields that are not equal.
    /// The function returns the NEW field value.
    fn differing_fields(&self, new: &Self) -> Vec<Field>;
}
