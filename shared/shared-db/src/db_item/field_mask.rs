use std::marker::PhantomData;

use ahash::AHashSet;

use crate::{DbAdaptor, DbItem};

/// Битовая маска полей [`DbItem`].
///
/// Каждый элемент `fields[i]` отвечает за поле `T::FIELDS[i]`.
/// `true` означает, что поле участвует в операции (SELECT, UPDATE, bind и т.д.).
///
/// Используется для частичных обновлений: вместо передачи `Vec<&str>`
/// с именами полей, которые нужно обновить, методы `bind_limited_fields`
/// и `update_masked` принимают маску -- это быстрее и не требует поиска по строкам.
#[derive(Debug, Clone)]
pub struct DbFieldMask<T> {
    fields: Vec<bool>,
    _phantom_data: PhantomData<T>,
}

impl<T> DbFieldMask<T>
where
    T: DbItem,
{
    /// Маска со всеми полями включёнными.
    pub fn all() -> Self {
        DbFieldMask {
            fields: vec![true; T::FIELDS.len()],
            _phantom_data: Default::default(),
        }
    }

    /// Маска с нулём включённых полей.
    pub fn none() -> Self {
        DbFieldMask {
            fields: vec![false; T::FIELDS.len()],
            _phantom_data: Default::default(),
        }
    }

    /// Маска только с указанными полями.
    pub fn with_fields<'a, I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = &'a S>,
        S: 'a + AsRef<str>,
    {
        let mut mask = DbFieldMask::none();
        mask.set_by_name(true, fields);
        mask
    }

    /// Добавляет к маске все первичные ключи.
    pub fn with_pkeys(mut self) -> Self {
        self.set_pkeys();
        self
    }

    /// Убирает из маски все GENERATED ALWAYS AS поля.
    pub fn without_autogen(mut self) -> Self {
        self.unset_always_autogen();
        self
    }

    /// Строит маску для операции `bind` в sqlx.
    ///
    /// Всегда включает первичные ключи, потому что они нужны для WHERE-части UPDATE.
    pub fn make_bind_mask<'a, I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = &'a S>,
        S: 'a + AsRef<str>,
    {
        Self::with_fields(fields).with_pkeys()
    }

    /// Маска для одиночного UPDATE: нужные поля + первичные ключи - autogen_always.
    ///
    /// Autogen_always поля (`GENERATED ALWAYS AS`) нельзя ставить в SET.
    pub fn make_single_update_bind_mask<'a, I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = &'a S>,
        S: 'a + AsRef<str>,
    {
        Self::with_fields(fields).with_pkeys().without_autogen()
    }

    /// Возвращает имена полей с `true` в маске.
    pub fn to_fields<I: FromIterator<&'static str>>(&self) -> I {
        std::iter::zip(T::FIELDS, &self.fields)
            .filter_map(|(f, m)| m.then_some(*f))
            .collect()
    }

    /// Возвращает инвертированную маску.
    pub fn inverted(&self) -> Self {
        Self {
            fields: self.fields.iter().map(|x| !*x).collect(),
            _phantom_data: Default::default(),
        }
    }

    /// Итерирует по булевым значениям маски.
    pub fn iter(&self) -> impl Iterator<Item = &bool> {
        self.fields.iter()
    }

    /// Устанавливает значение `value` для всех полей из списка `fields` по имени.
    fn set_by_name<'a, I, S>(&mut self, value: bool, fields: I)
    where
        I: IntoIterator<Item = &'a S>,
        S: 'a + AsRef<str>,
    {
        let fields = fields.into_iter().map(AsRef::as_ref).collect::<AHashSet<_>>();
        for (idx, field) in T::FIELDS.iter().enumerate() {
            if fields.contains(field) {
                self.fields[idx] = value;
            }
        }
    }

    /// Устанавливает значение `value` для полей по индексам.
    ///
    /// Использовать только с индексами из констант макроса (`T::PRIMARY_KEY_INDICES`,
    /// `T::NO_UPDATE_INDICES`), иначе возможна паника из-за выхода за границы.
    fn set(&mut self, value: bool, indices: &[usize]) {
        for i in indices {
            self.fields[*i] = value;
        }
    }

    /// Включает все первичные ключи в маску.
    fn set_pkeys(&mut self) {
        self.set(true, T::PRIMARY_KEY_INDICES);
    }

    /// Выключает все autogen_always поля из маски.
    fn unset_always_autogen(&mut self) {
        self.set(false, T::NO_UPDATE_INDICES);
    }
}

impl<T> std::ops::Index<usize> for DbFieldMask<T> {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        &self.fields[index]
    }
}

impl<T> std::ops::IndexMut<usize> for DbFieldMask<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.fields[index]
    }
}

/// `|` объединяет две маски: поле включается если оно включено хотя бы в одной.
impl<T> std::ops::BitOr for DbFieldMask<T> {
    type Output = DbFieldMask<T>;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        std::iter::zip(&mut self.fields, rhs.fields)
            .for_each(|(l, r)| *l = *l || r);
        self
    }
}

impl<T> PartialEq for DbFieldMask<T> {
    fn eq(&self, other: &Self) -> bool {
        self.fields == other.fields
    }
}

impl<T, const N: usize> PartialEq<[bool; N]> for DbFieldMask<T> {
    fn eq(&self, other: &[bool; N]) -> bool {
        self.fields.as_slice() == &other[..]
    }
}

impl<T> PartialEq<[bool]> for DbFieldMask<T> {
    fn eq(&self, other: &[bool]) -> bool {
        self.fields.as_slice() == other
    }
}

/// Маска полей для [`DbAdaptor`].
///
/// Содержит два компонента:
/// - `item_field_mask` -- маска для полей базового [`DbItem`]
/// - `dup_field_mask` -- маска для полей-дубликатов (`DUP_FIELDS`)
///
/// Разделение нужно потому что адаптор может содержать поля, не присутствующие
/// в таблице БД (дубликаты с другим именем для фронтенда).
pub struct DbAdaptorFieldMask<T: DbAdaptor> {
    item_field_mask: DbFieldMask<T::DbItem>,
    dup_field_mask: Vec<bool>,
}

impl<T: DbAdaptor> DbAdaptorFieldMask<T> {
    /// Все поля включены.
    pub fn all() -> Self {
        DbAdaptorFieldMask {
            item_field_mask: DbFieldMask::all(),
            dup_field_mask: vec![true; T::DUP_FIELDS.len()],
        }
    }

    /// Ни одного поля не включено.
    pub fn none() -> Self {
        DbAdaptorFieldMask {
            item_field_mask: DbFieldMask::none(),
            dup_field_mask: vec![false; T::DUP_FIELDS.len()],
        }
    }

    /// Включает только указанные поля.
    pub fn with_fields<'a, I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = &'a S>,
        S: 'a + AsRef<str>,
    {
        // Собираем AHashSet один раз: он используется и для DUP_FIELDS и для
        // основных полей DbItem, не тратя лишней памяти при оптимизации компилятором.
        let fields = fields.into_iter().map(AsRef::as_ref).collect::<AHashSet<_>>();
        let dup_field_mask =
            T::DUP_FIELDS.iter().map(|f| fields.contains(f)).collect();
        let item_field_mask = DbFieldMask::with_fields(&fields);
        DbAdaptorFieldMask {
            item_field_mask,
            dup_field_mask,
        }
    }

    /// Включает указанные поля плюс первичные ключи.
    ///
    /// Первичные ключи нужны почти всегда при конвертации в адаптор, чтобы
    /// клиент знал идентификатор записи даже когда остальные поля не запрошены.
    pub fn with_fields_and_pkeys<'a, I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = &'a S>,
        S: 'a + AsRef<str>,
    {
        let fields = fields.into_iter().map(AsRef::as_ref).collect::<AHashSet<_>>();
        let dup_field_mask =
            T::DUP_FIELDS.iter().map(|f| fields.contains(f)).collect();
        let item_field_mask = DbFieldMask::with_fields(&fields).with_pkeys();
        DbAdaptorFieldMask {
            item_field_mask,
            dup_field_mask,
        }
    }

    pub fn item_field_mask(&self) -> &DbFieldMask<T::DbItem> {
        &self.item_field_mask
    }

    pub fn dup_field_mask(&self) -> &[bool] {
        &self.dup_field_mask
    }
}
