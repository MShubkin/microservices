//! This is meant to simulate the number range operation from
//! the original SAP system. Initially it will be used locally as is,
//! but eventually a decision will have to be made as to whether it matches
//! our requirements or not.
#![allow(dead_code)]
use crate::common::{ProcessingError, Result};

use asez2_shared_db::db_item::{DbItem, Select, SelectionKind};
use asez2_shared_db::Value;
use shared_essential::application::records::Recorder;
use sqlx::Type;
use sqlx::{Postgres, Transaction};

#[derive(Debug, Clone, PartialEq, DbItem)]
#[item_table = "number_range"]
pub(crate) struct NumberRange {
    #[item_field_pkey]
    object_type: EcObjectType,
    start_idx: i64,
    end_idx: i64,
    next_idx: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NumberRequest {
    kind: EcObjectType,
    count: usize,
}

impl NumberRequest {
    pub(crate) fn new(kind: EcObjectType, count: usize) -> Self {
        Self { kind, count }
    }
}

/// This performs a transaction where:
/// 1. Numbers are generated.
/// 2. Some DB operation is performed, ideally using those numbers.
/// 3. If the operation is performed which returns a list of inserted items.
/// 4. If the operation fails, the operation is rolled back.
///
/// It is assumed that the operation involves inserting the items that use those numbers,
/// however, strictly speaking this is not necessarily so.
///
/// NB: The number of items fed into the closure should be the same as count. If it is not
/// then we return an error.
pub(crate) async fn op_with_numbers<'a, Output, FutFn>(
    mut recorder: Recorder<'a>,
    counts: Vec<NumberRequest>,
    op: FutFn,
) -> Result<Output>
where
    FutFn: for<'b> FnOnce(
            ahash::AHashMap<EcObjectType, Vec<i64>>,
            &'b mut Recorder<'a>,
        ) -> futures::future::BoxFuture<'b, Result<Output>>
        + Send,
{
    let numbers = get_next_numbers(recorder.tx(), counts).await?;

    // TODO: Solve the lifetime problem.
    let output = op(numbers, &mut recorder).await?;

    recorder.commit().await?;
    Ok(output)
}

pub(super) async fn get_next_numbers(
    p: &mut Transaction<'_, Postgres>,
    counts: Vec<NumberRequest>,
) -> Result<ahash::AHashMap<EcObjectType, Vec<i64>>> {
    let s = Select::with_fields(NumberRange::FIELDS).add_expand_filter(
        "object_type",
        SelectionKind::In,
        counts.iter().map(|x| Value::from(x.kind as i16)).collect::<Vec<_>>(),
    );
    let mut range = NumberRange::select(&s, &mut *p).await?;

    let mut ret = ahash::AHashMap::new();
    for r in range.iter_mut() {
        if let Some(NumberRequest { count, kind }) =
            counts.iter().find(|x| x.kind == r.object_type)
        {
            let start = r.next_idx;
            r.next_idx += *count as i64;
            // This check should be done when inserting rather than retrieving for
            // more "watertight" code.
            if r.next_idx - 1 > r.end_idx {
                return Err(ProcessingError::NumberRangeOverflow(*kind));
            }

            ret.insert(*kind, (start..r.next_idx).collect::<Vec<_>>());
        }
    }
    NumberRange::update_vec(&range, Some(&["next_idx"]), p).await?;
    Ok(ret)
}

/// This is an enum that corresponds to object type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Type)]
#[repr(i16)]
pub enum EcObjectType {
    Undefined = 0,
    Agenda = 1,
    Protocol = 2,
}

impl Default for EcObjectType {
    fn default() -> Self {
        Self::Undefined
    }
}

#[derive(Debug, Clone)]
struct NumbersError(String);

impl std::fmt::Display for NumbersError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for NumbersError {}

impl From<i16> for EcObjectType {
    fn from(i: i16) -> Self {
        match i {
            1 => Self::Agenda,
            2 => Self::Protocol,
            _ => Self::Undefined,
        }
    }
}
