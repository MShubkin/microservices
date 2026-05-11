#![doc = include_str!("filtering.md")]

use std::collections::hash_map::Entry;

use ahash::AHashMap;
use asez2_shared_db::db_item::{Filter, FilterTree, SelectionKind};
use shared_essential::{
    domain::routes::{CritValue, TryFromValueError},
    presentation::dto::master_data::{error::MasterDataError, request::CritArg},
};

#[derive(Debug, thiserror::Error)]
pub(super) enum CriteriaFilterError {
    #[error("неподдерживаемая комбинация фильтров")]
    InvalidStructure,
    #[error("неподдерживаемый тип фильтра `{0}`")]
    InvalidKind(SelectionKind),
    #[error("не поддерживается фильтр `OR` для различных полей `{0}` и `{1}`")]
    InvalidDisjunct(String, String),
    #[error("конфликтующие фильтры для поля `{0}`")]
    Conflict(String),
    #[error(transparent)]
    Value(#[from] TryFromValueError),
}

impl From<CriteriaFilterError> for MasterDataError {
    fn from(error: CriteriaFilterError) -> Self {
        MasterDataError::InternalError(format!(
            "ошибка обратотки фильтров критериев: {error}"
        ))
    }
}

pub(super) type CriteriaFilterResult<T> = Result<T, CriteriaFilterError>;

/// Преобразует фильтры полей с конкретными значениями (Eq, In)
/// в отображение имен полей в аргументы критериев, подходящее для
/// поиска маршрутов.
pub(super) fn filters_to_values(
    filter_tree: FilterTree,
) -> CriteriaFilterResult<AHashMap<String, CritArg>> {
    let mut res = AHashMap::new();
    filter_to_values_inner(vec![filter_tree], &mut res)?;
    Ok(res)
}

fn filter_to_values_inner(
    filter_trees: Vec<FilterTree>,
    res: &mut AHashMap<String, CritArg>,
) -> CriteriaFilterResult<()> {
    filter_trees.into_iter().try_for_each(|ft| {
        match ft {
            FilterTree::Or(fts) => {
                if let Some((field, values)) = or_filters_to_values(fts)? {
                    append_values(res, field, values)?;
                }
            }
            FilterTree::And(fts) => filter_to_values_inner(fts, res)?,
            FilterTree::Filter(filter) => {
                let (field, values) = filter_to_values(filter)?;
                append_values(res, field, CritArg::AnyAgg(values))?;
            }
            FilterTree::None => {}
        }
        Ok(())
    })
}

fn append_values(
    map: &mut AHashMap<String, CritArg>,
    field: String,
    values: CritArg,
) -> CriteriaFilterResult<()> {
    let entry = map.entry(field);
    if matches!(entry, Entry::Occupied(_)) {
        return Err(CriteriaFilterError::Conflict(entry.key().clone()));
    }
    entry.or_insert(values);
    Ok(())
}

fn or_filters_to_values(
    filter_trees: Vec<FilterTree>,
) -> CriteriaFilterResult<Option<(String, CritArg)>> {
    let maybe_field_values = filter_trees.into_iter().try_fold(
        None,
        |acc: Option<(String, Vec<CritValue>)>, ft| {
            let (field, values) = match ft {
                FilterTree::None => return Ok(acc),
                FilterTree::Filter(filter) => filter_to_values(filter)?,
                _ => return Err(CriteriaFilterError::InvalidStructure),
            };
            match acc {
                Some((f, _)) if f != field => {
                    Err(CriteriaFilterError::InvalidDisjunct(f, field))
                }
                Some((f, mut v)) => {
                    v.extend(values);
                    Ok(Some((f, v)))
                }
                None => Ok(Some((field, values))),
            }
        },
    )?;
    Ok(maybe_field_values.map(|(field, values)| (field, CritArg::AnyAgg(values))))
}

fn filter_to_values(
    filter: Filter,
) -> CriteriaFilterResult<(String, Vec<CritValue>)> {
    let Filter {
        field,
        kind,
        values,
    } = filter;
    match kind {
        SelectionKind::Equals | SelectionKind::In => {
            let values = values
                .into_iter()
                .map(CritValue::try_from)
                .collect::<Result<_, _>>()?;
            Ok((field, values))
        }
        _ => Err(CriteriaFilterError::InvalidKind(kind)),
    }
}

#[cfg(test)]
mod tests {
    use asez2_shared_db::db_item::{Filter, FilterTree, Select};
    use shared_essential::{
        domain::routes::CritValue, presentation::dto::master_data::request::CritArg,
    };

    #[test]
    fn filters_to_values() {
        let select =
            Select::default().eq("field_eq", 10).in_any("field_in", [1, 2, 3]);
        let values = super::filters_to_values(select.filter_list).expect("ok");

        assert_eq!(
            values.get("field_eq").expect("should exist"),
            &CritArg::AnyAgg(vec![CritValue::Int(10)])
        );

        assert_eq!(
            values.get("field_in").expect("should exist"),
            &CritArg::AnyAgg(vec![
                CritValue::Int(1),
                CritValue::Int(2),
                CritValue::Int(3)
            ])
        );
    }

    #[test]
    fn many_fields() {
        const N: i64 = 16;

        let filter = FilterTree::or_from_list([
            Filter::eq("field", 100),
            Filter::eq("field", 200),
            Filter::eq("field", 300),
        ]);
        let filter_list = (0..N).fold(filter, |filter, n| {
            filter.and(Filter::eq(&format!("field{n}"), n).into())
        });

        let values = super::filters_to_values(filter_list).expect("ok");

        assert_eq!(values.len(), N as usize + 1);
        assert_eq!(
            values.get("field"),
            Some(&CritArg::AnyAgg(vec![
                CritValue::Int(100),
                CritValue::Int(200),
                CritValue::Int(300),
            ]))
        );
        for n in 0..N {
            assert_eq!(
                values.get(&format!("field{n}")),
                Some(&CritArg::AnyAgg(vec![CritValue::Int(n)]))
            );
        }
    }
}
