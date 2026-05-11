//! Модуль под маппинг значений одних полей для других
//! в зависимости от секции
//!
//! Например для запроса по секции СК по contract_amendment требуется
//! получать значение из delta_sum_excluded_vat и записывать
//! как sum_excluded_vat
use asez2_shared_db::db_item::Select;
use asez2_shared_db::DbAdaptor;
use shared_essential::domain::{
    ContractAmendment, ContractAmendmentRep, PlanOrAmendment, PlanOrAmendmentRep,
    PlanRep, SectionKind,
};

/// Маппинг полей в зависимости от секции
#[allow(dead_code)]
pub(crate) trait SectionMap {
    /// (откуда взять значение, куда положить значение, вид секции)
    const MAPPINGS: &'static [(&'static str, &'static str, SectionKind)];

    /// Заполнение данных из одних полей в другие и очищение значений для полей, откуда берутся значения.
    /// Поля заполняются только в том случае, если `fields` содержит эти самые поля
    fn apply_section_mappings<T>(
        self,
        section_kind: SectionKind,
        fields: &[T],
    ) -> Self
    where
        T: AsRef<str>;

    /// Преобразование [`DbAdaptor::DbItem`] в его репрезентацию с применением
    /// маппингов по секции
    fn from_item_with_section_mapping<T>(
        item: Self::DbItem,
        section_kind: SectionKind,
        fields: Option<&[T]>,
    ) -> Self
    where
        Self: DbAdaptor,
        T: AsRef<str>,
    {
        let adaptor = Self::from_item(item, fields);
        adaptor.apply_section_mappings(section_kind, fields.unwrap_or_default())
    }

    /// Обогащение селекта доп полями, чтобы данные по этим полям тоже были
    /// возвращены для дальнейшего маппинга
    fn enrich_select_by_section(select: &mut Select, section_kind: SectionKind) {
        for i in 0..select.field_list.len() {
            let field = &select.field_list[i];

            if let Some(mapping) = Self::field_mapping(field, section_kind) {
                select.field_list.push(mapping.to_owned())
            }
        }

        select.filter_list.slice_mut().iter_mut().for_each(|filter| {
            if let Some(mapping) = Self::field_mapping(&filter.field, section_kind)
            {
                filter.field = mapping.to_owned()
            }
        });

        select.order_list.iter_mut().for_each(|ordering| {
            if let Some(mapping) =
                Self::field_mapping(&ordering.field, section_kind)
            {
                ordering.field = mapping.to_owned()
            }
        });
    }

    /// Возвращение маппингов по определенному типу секции
    fn section_mappings(
        section_kind: SectionKind,
    ) -> Vec<(&'static str, &'static str)> {
        Self::MAPPINGS
            .iter()
            .filter(|(_, _, kind)| section_kind == *kind)
            .map(|(from, to, _)| (*from, *to))
            .collect()
    }

    /// Возможный маппинг по полю для определенного типа секции
    fn field_mapping(
        field: &str,
        section_kind: SectionKind,
    ) -> Option<&'static str> {
        Self::MAPPINGS.iter().find_map(|(from, to, kind)| {
            (to == &field && section_kind == *kind).then_some(*from)
        })
    }

    /// Содержит ли массив полей поле, которое имеет маппинг
    fn contains_mapping_field<T>(
        fields: &[T],
        field: &str,
        section_kind: SectionKind,
    ) -> bool
    where
        T: AsRef<str>,
    {
        Self::field_mapping(field, section_kind)
            .map(|_| fields.iter().any(|f| f.as_ref() == field))
            .unwrap_or(false)
    }
}

pub(crate) trait SectionMapExt: SectionMap {
    type Item;

    fn from_item_with_section_mapping<T>(
        item: Self::Item,
        section_kind: SectionKind,
        fields: Option<&[T]>,
    ) -> Self
    where
        Self: Sized,
        T: AsRef<str>;
}

impl SectionMap for PlanOrAmendmentRep {
    const MAPPINGS: &'static [(&'static str, &'static str, SectionKind)] =
        &[ContractAmendmentRep::MAPPINGS[0], ContractAmendmentRep::MAPPINGS[1]];

    fn apply_section_mappings<T>(
        self,
        section_kind: SectionKind,
        fields: &[T],
    ) -> Self
    where
        T: AsRef<str>,
    {
        match self {
            Self::Plan(p) => {
                Self::Plan(p.apply_section_mappings(section_kind, fields))
            }
            Self::Amendment(a) => {
                Self::Amendment(a.apply_section_mappings(section_kind, fields))
            }
        }
    }
}

impl SectionMapExt for PlanOrAmendmentRep {
    type Item = PlanOrAmendment;

    fn from_item_with_section_mapping<T>(
        item: Self::Item,
        section_kind: SectionKind,
        fields: Option<&[T]>,
    ) -> Self
    where
        T: AsRef<str>,
    {
        let adaptor = Self::from_item(item, fields);
        adaptor.apply_section_mappings(section_kind, fields.unwrap_or_default())
    }
}

impl SectionMap for ContractAmendmentRep {
    const MAPPINGS: &'static [(&'static str, &'static str, SectionKind)] = &[
        (
            ContractAmendment::delta_sum_excluded_vat,
            ContractAmendment::sum_excluded_vat,
            SectionKind::EstimatedCommission,
        ),
        (
            ContractAmendment::pricing_delta_sum_excluded_vat,
            ContractAmendment::pricing_sum_excluded_vat,
            SectionKind::EstimatedCommission,
        ),
    ];

    fn apply_section_mappings<T>(
        mut self,
        section_kind: SectionKind,
        fields: &[T],
    ) -> Self
    where
        T: AsRef<str>,
    {
        if section_kind == SectionKind::EstimatedCommission {
            Self::contains_mapping_field(
                fields,
                ContractAmendment::sum_excluded_vat,
                section_kind,
            )
            .then(|| {
                self.sum_excluded_vat = self.delta_sum_excluded_vat;
                self.delta_sum_excluded_vat = None;
            });
            Self::contains_mapping_field(
                fields,
                ContractAmendment::pricing_sum_excluded_vat,
                section_kind,
            )
            .then(|| {
                self.pricing_sum_excluded_vat =
                    self.pricing_delta_sum_excluded_vat.flatten();
                self.pricing_delta_sum_excluded_vat = None;
            });
        }

        self
    }
}

impl SectionMap for PlanRep {
    const MAPPINGS: &'static [(&'static str, &'static str, SectionKind)] = &[];

    fn apply_section_mappings<T>(
        self,
        _section_kind: SectionKind,
        _fields: &[T],
    ) -> Self
    where
        T: AsRef<str>,
    {
        self
    }
}
