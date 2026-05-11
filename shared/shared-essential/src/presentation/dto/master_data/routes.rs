use std::num::TryFromIntError;

use asez2_tables::master_data::routes::*;
use serde::{Deserialize, Serialize};

use crate::presentation::dto::value::UiValue;

use super::response::RouteDetailsResponse;

#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    derive_more::Display,
    Clone,
    Copy,
)]
#[serde(rename_all = "snake_case")]
pub enum RouteCriterionOperator {
    /// Не задан
    #[default]
    #[display(fmt = "Не задан")]
    Undefined,
    /// Равно. Symbol "="
    #[display(fmt = "=")]
    Equal,
    /// Не Равно. Symbol "!="
    #[display(fmt = "!=")]
    NotEqual,
    /// Symbol "<"
    #[display(fmt = "<")]
    Less,
    /// Symbol "<="
    #[display(fmt = "<=")]
    LessEqual,
    /// Symbol ">"
    #[display(fmt = ">")]
    Greater,
    /// Symbol ">="
    #[display(fmt = ">=")]
    GreaterEqual,
    /// Between
    #[display(fmt = "between")]
    Between,
    /// In
    #[display(fmt = "in")]
    In,
    /// Symbol *
    #[display(fmt = "*")]
    All,
    /// Поддерево многоуровневого справочника.
    InTree,
}

#[derive(Debug, thiserror::Error)]
#[error("значение `{0}` не может быть использовано для критериев маршрутов")]
pub struct TryFromUiValueError(String);

impl TryFrom<UiValue> for CritValue {
    type Error = TryFromUiValueError;

    fn try_from(value: UiValue) -> Result<Self, Self::Error> {
        match value {
            UiValue::String(s) => Ok(CritValue::String(s)),
            UiValue::Int(i) => Ok(CritValue::Int(i)),
            UiValue::Bool(b) => Ok(CritValue::Bool(b)),
            UiValue::Date(d) => Ok(CritValue::Date(d)),
            UiValue::Timestamp(t) => Ok(CritValue::Timestamp(t)),
            _ => Err(TryFromUiValueError(format!("{value:?}"))),
        }
    }
}

impl TryFrom<&UiValue> for CritValue {
    type Error = TryFromUiValueError;

    fn try_from(value: &UiValue) -> Result<Self, Self::Error> {
        match value {
            UiValue::String(s) => Ok(CritValue::String(s.clone())),
            UiValue::Int(i) => Ok(CritValue::Int(*i)),
            UiValue::Bool(b) => Ok(CritValue::Bool(*b)),
            UiValue::Date(d) => Ok(CritValue::Date(*d)),
            UiValue::Timestamp(t) => Ok(CritValue::Timestamp(*t)),
            _ => Err(TryFromUiValueError(format!("{value:?}"))),
        }
    }
}

impl From<CritValue> for UiValue {
    fn from(value: CritValue) -> Self {
        match value {
            CritValue::String(s) => UiValue::String(s),
            CritValue::Int(i) => UiValue::Int(i),
            CritValue::Bool(b) => UiValue::Bool(b),
            CritValue::Date(d) => UiValue::Date(d),
            CritValue::Timestamp(t) => UiValue::Timestamp(t),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct RouteCriterion {
    pub operator: RouteCriterionOperator,
    pub filter_values: Vec<UiValue>,
}

#[derive(Debug, thiserror::Error)]
pub enum TryFromRouteCritError {
    #[error("Неизвестный предикат")]
    Unknown,
    #[error("Неподдерживаемый предикат {0:?}")]
    Unsupported(CritPredicate),
    #[error("Ошибка преобразования")]
    Conv,
}

#[derive(Debug, thiserror::Error)]
pub enum TryFromRouteCriterionError {
    #[error("Неверное значение `values`: {0}")]
    InvalidValues(&'static str),
    #[error("Недопустимый тип критерия: {0}")]
    Unsupported(RouteCriterionOperator),
    #[error("Ошибка преобразования: {0}")]
    Conv(#[from] TryFromUiValueError),
    #[error("Ошибка преобразования: {0}")]
    ConvInt(#[from] TryFromIntError),
}

pub fn try_into_route_criteria(
    predicate: CritPredicate,
) -> Result<Vec<RouteCriterion>, TryFromRouteCritError> {
    match predicate {
        CritPredicate::Or { predicates } => Ok(predicates
            .into_iter()
            .map(try_into_route_criterion)
            .collect::<Result<_, _>>()?),
        CritPredicate::None => Ok(vec![]),
        _ => Ok(vec![try_into_route_criterion(predicate)?]),
    }
}

fn try_into_route_criterion(
    predicate: CritPredicate,
) -> Result<RouteCriterion, TryFromRouteCritError> {
    let (kind, values) = match predicate {
        CritPredicate::Unknown => return Err(TryFromRouteCritError::Unknown),
        CritPredicate::Equal { value } => {
            (RouteCriterionOperator::Equal, vec![value])
        }
        CritPredicate::NotEqual { value } => {
            (RouteCriterionOperator::NotEqual, vec![value])
        }
        CritPredicate::Less { value } => {
            (RouteCriterionOperator::Less, vec![value])
        }
        CritPredicate::LessEqual { value } => {
            (RouteCriterionOperator::LessEqual, vec![value])
        }
        CritPredicate::Greater { value } => {
            (RouteCriterionOperator::Greater, vec![value])
        }
        CritPredicate::GreaterEqual { value } => {
            (RouteCriterionOperator::GreaterEqual, vec![value])
        }
        CritPredicate::Between { low, high } => {
            (RouteCriterionOperator::Between, vec![low, high])
        }
        CritPredicate::Any => (RouteCriterionOperator::All, vec![]),
        CritPredicate::In { values } => (RouteCriterionOperator::In, values),
        CritPredicate::InTree {
            dictionary: _,
            roots,
        } => (
            RouteCriterionOperator::InTree,
            roots.into_iter().map(|x| CritValue::Int(x.into())).collect(),
        ),
        CritPredicate::Not { .. }
        | CritPredicate::And { .. }
        | CritPredicate::Or { .. }
        | CritPredicate::None => {
            return Err(TryFromRouteCritError::Unsupported(predicate))
        }
    };

    let values = values
        .into_iter()
        .map(TryFrom::try_from)
        .collect::<Result<_, _>>()
        .map_err(|_| TryFromRouteCritError::Conv)?;
    Ok(RouteCriterion {
        operator: kind,
        filter_values: values,
    })
}

fn single_value(
    values: Vec<UiValue>,
) -> Result<CritValue, TryFromRouteCriterionError> {
    let value = asez2_shared_db::value::single_value(values)
        .map_err(TryFromRouteCriterionError::InvalidValues)?;
    Ok(value.try_into()?)
}

fn two_values(
    values: Vec<UiValue>,
) -> Result<(CritValue, CritValue), TryFromRouteCriterionError> {
    let (value1, value2) = asez2_shared_db::value::two_values(values)
        .map_err(TryFromRouteCriterionError::InvalidValues)?;
    Ok((value1.try_into()?, value2.try_into()?))
}

pub fn try_from_route_criteria(
    field_name: &str,
    mut values: Vec<RouteCriterion>,
) -> Result<CritPredicate, TryFromRouteCriterionError> {
    let try_from_route_criterion =
        |crit| try_from_route_criterion(field_name, crit);
    let single = values.pop();
    match (single, values.is_empty()) {
        (None, _) => Ok(CritPredicate::None),
        (Some(crit), true) => try_from_route_criterion(crit),
        (Some(crit), _) => Ok(CritPredicate::Or {
            predicates: values
                .into_iter()
                .chain(std::iter::once(crit))
                .map(try_from_route_criterion)
                .collect::<Result<_, _>>()?,
        }),
    }
}

fn try_from_route_criterion(
    field_name: &str,
    value: RouteCriterion,
) -> Result<CritPredicate, TryFromRouteCriterionError> {
    let RouteCriterion {
        operator: kind,
        filter_values: values,
    } = value;
    let predicate = match kind {
        RouteCriterionOperator::Equal => CritPredicate::Equal {
            value: single_value(values)?,
        },
        RouteCriterionOperator::NotEqual => CritPredicate::NotEqual {
            value: single_value(values)?,
        },
        RouteCriterionOperator::Less => CritPredicate::Less {
            value: single_value(values)?,
        },
        RouteCriterionOperator::LessEqual => CritPredicate::LessEqual {
            value: single_value(values)?,
        },
        RouteCriterionOperator::Greater => CritPredicate::Greater {
            value: single_value(values)?,
        },
        RouteCriterionOperator::GreaterEqual => CritPredicate::GreaterEqual {
            value: single_value(values)?,
        },
        RouteCriterionOperator::Between => {
            let (low, high) = two_values(values)?;
            CritPredicate::Between { low, high }
        }
        RouteCriterionOperator::In => CritPredicate::In {
            values: values
                .into_iter()
                .map(CritValue::try_from)
                .collect::<Result<_, _>>()?,
        },
        RouteCriterionOperator::InTree => CritPredicate::InTree {
            dictionary: field_name
                .strip_suffix("_id")
                .unwrap_or(field_name)
                .to_string(),
            roots: values
                .into_iter()
                .map(|x| match x.try_into()? {
                    CritValue::Int(value) => Ok(value.try_into()?),
                    _ => Err(TryFromRouteCriterionError::InvalidValues(
                        "для in_tree допускаются только целочисленные значения",
                    )),
                })
                .collect::<Result<_, _>>()?,
        },
        RouteCriterionOperator::Undefined => {
            return Err(TryFromRouteCriterionError::Unsupported(kind))
        }
        RouteCriterionOperator::All => CritPredicate::Any,
    };
    Ok(predicate)
}

impl TryFrom<(RouteHeaderRep, Vec<RouteCrit>, Option<RouteDataContent>)>
    for RouteDetailsResponse
{
    type Error = TryFromRouteCritError;
    fn try_from(
        (header, criteria, data): (
            RouteHeaderRep,
            Vec<RouteCrit>,
            Option<RouteDataContent>,
        ),
    ) -> Result<Self, Self::Error> {
        let criteria_set = criteria
            .into_iter()
            .map(|crit| {
                Ok((crit.field_name, try_into_route_criteria(crit.predicate.0)?))
            })
            .collect::<Result<_, _>>()?;
        let result = RouteDetailsResponse {
            header,
            criteria_set,
            data,
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use asez2_tables::master_data::routes::{
        CritPredicate, CritValue, RouteApprType, RouteCrit, RouteDataContent,
        RouteHeaderRep,
    };
    use sqlx::types::Json;

    use crate::presentation::dto::master_data::response::RouteDetailsResponse;

    use super::*;

    #[test]
    fn try_into_ui_route() {
        let route = RouteHeaderRep {
            type_id: Some(RouteApprType::SpecializedDepartments),
            route_id: Some(2),
            name_short: Some(Some("route name".to_string())),
            ..Default::default()
        };
        let crits = vec![
            RouteCrit {
                field_name: "f1".to_string(),
                predicate: Json(CritPredicate::Or {
                    predicates: vec![
                        CritPredicate::Equal {
                            value: CritValue::Int(10),
                        },
                        CritPredicate::Between {
                            low: CritValue::Int(20),
                            high: CritValue::Int(30),
                        },
                    ],
                }),
                ..Default::default()
            },
            RouteCrit {
                field_name: "f2".to_string(),
                predicate: Json(CritPredicate::NotEqual {
                    value: CritValue::Int(40),
                }),
                ..Default::default()
            },
            RouteCrit {
                field_name: "f3".to_string(),
                predicate: Json(CritPredicate::Any),
                ..Default::default()
            },
            RouteCrit {
                field_name: "f4".to_string(),
                predicate: Json(CritPredicate::None),
                ..Default::default()
            },
        ];
        let data = Some(RouteDataContent::AssignDepartment(Default::default()));

        let route_ui = RouteDetailsResponse::try_from((route, crits, data))
            .expect("should succeed");
        let f1 = route_ui.criteria_set.get("f1").expect("f1 should exist");
        let f2 = route_ui.criteria_set.get("f2").expect("f2 should exist");
        let f3 = route_ui.criteria_set.get("f3").expect("f3 should exist");
        let f4 = route_ui.criteria_set.get("f4").expect("f4 should exist");
        assert_eq!(f1.len(), 2);
        assert_eq!(f2.len(), 1);
        assert_eq!(f3.len(), 1);
        assert_eq!(f4.len(), 0);
    }

    #[test]
    fn try_from_ui_route() {
        let criteria1 = vec![RouteCriterion {
            operator: RouteCriterionOperator::All,
            filter_values: vec![],
        }];
        let criteria2 = vec![
            RouteCriterion {
                operator: RouteCriterionOperator::Equal,
                filter_values: vec![UiValue::Int(10)],
            },
            RouteCriterion {
                operator: RouteCriterionOperator::Equal,
                filter_values: vec![UiValue::Int(20)],
            },
        ];
        let criteria3 = vec![];

        let act1 = try_from_route_criteria("f1", criteria1).expect("ok 1");
        let exp1 = CritPredicate::Any;
        assert_eq!(act1, exp1);

        let act2 = try_from_route_criteria("f2", criteria2).expect("ok 2");
        let exp2 = CritPredicate::Or {
            predicates: vec![
                CritPredicate::Equal {
                    value: CritValue::Int(10),
                },
                CritPredicate::Equal {
                    value: CritValue::Int(20),
                },
            ],
        };
        assert_eq!(act2, exp2);

        let act3 = try_from_route_criteria("f3", criteria3).expect("ok 3");
        let exp3 = CritPredicate::None;
        assert_eq!(act3, exp3);
    }
}
