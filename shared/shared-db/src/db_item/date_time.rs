//! Обёртки над типами дат и времени для [`DbItem`].
//!
//! [`sqlx::types::time::Date`] и [`sqlx::types::time::PrimitiveDateTime`] не реализуют
//! [`Default`], поэтому их нельзя использовать напрямую в структурах, которые
//! требуют `#[derive(Default)]` для работы с `activate_fields`.
//!
//! `AsezDate` и `AsezTimestamp` решают эту проблему: они оборачивают оригинальные типы
//! и реализуют `Default` как `1901-01-01` -- условную "пустую" дату в системе.
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use sqlx::{
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef},
    types::{
        chrono::{DateTime, NaiveDate, NaiveDateTime, Utc},
        time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset},
    },
    Decode, Encode, Postgres, Type,
};
use std::{fmt::Display, ops::Deref, time::Duration};
use time::Weekday;

/// Формат дат в БД (`YYYY-MM-DD`).
pub(crate) const DB_DATE_FORMAT: &str = "%Y-%m-%d";
/// Формат временных меток в БД (`YYYY-MM-DD HH:MM:SS`).
pub(crate) const DB_TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Формат, в котором реализован [`Display`] для [`AsezDate`]
pub const DATE_FORMAT: &str = "%d.%m.%Y";
/// Формат, в котором реализован [`Display`] для [`AsezTimestamp`]
pub const TIMESTAMP_FORMAT: &str = "%d.%m.%Y %H:%M:%S";

/// Обёртка над [`sqlx::types::time::Date`] с реализацией `Default`.
///
/// Нужна для полей структур [`DbItem`], которые должны иметь дефолтное значение.
/// Сериализуется как строка вида `"31.12.2024"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AsezDate(pub Date);

impl AsezDate {
    pub fn today() -> Self {
        let date = OffsetDateTime::now_utc().date();
        AsezDate(date)
    }

    pub fn try_from_yo(year: i32, day: u16) -> Result<Self, String> {
        Date::try_from_yo(year, day).map(AsezDate).map_err(|_| {
            format!("{} год с {} днем не может быть валидной датой", year, day)
        })
    }

    pub fn number_days_from_monday(self) -> u8 {
        self.0.weekday().number_days_from_monday()
    }

    /// Возвращает ближайший следующий день недели `weekday` после этой даты.
    ///
    /// Если текущий день уже совпадает с `weekday`, возвращает следующую
    /// неделю (через 7 дней), а не саму эту дату.
    pub fn with_next_weekday(&self, weekday: Weekday) -> Self {
        let maybe_days_to_next_weekday = (weekday.number_days_from_monday() + 7
            - self.number_days_from_monday())
            % 7;

        let days_to_next_weekday = if maybe_days_to_next_weekday == 0 {
            7
        } else {
            maybe_days_to_next_weekday
        };

        AsezDate(
            self.0
                + Duration::from_secs(days_to_next_weekday as u64 * 24 * 60 * 60),
        )
    }

    /// Проверяет, равна ли дата дефолтному значению (`1901-01-01`).
    pub fn is_empty(&self) -> bool {
        *self == AsezDate::default()
    }

    /// Возвращает номер месяца или 0 для дефолтной даты.
    pub fn get_month(&self) -> u8 {
        if *self == AsezDate::default() {
            0
        } else {
            self.month()
        }
    }

    /// Конвертирует дату в число Excel (дни с 01.01.1900 + 2 из-за бага Lotus 1-2-3).
    pub fn to_excel(&self) -> f64 {
        if *self == AsezDate::default() {
            return 0.0;
        }

        let excel_zero_date = Date::try_from_ymd(1900, 1, 1).unwrap();
        let duration = self.0 - excel_zero_date;
        (duration.whole_days() + 2) as f64
    }

    pub fn from_julian_day(julian_date: i64) -> Self {
        Self(Date::from_julian_day(julian_date))
    }

    /// Считает количество рабочих дней от `self` до `end`, включая обе граничные даты.
    pub fn working_days_between(&self, end: AsezDate) -> u16 {
        let start_date = self.0;
        let end_date = end.0;

        if start_date == end_date {
            return if start_date.weekday().number_days_from_monday() < 5 {
                1
            } else {
                0
            };
        }

        // Полное количество дней и полные недели между датами
        let total_days = (end_date - start_date).whole_days();
        let full_weeks_working_days = (total_days / 7) * 5;

        // Остаток дней после вычитания полных недель
        let remaining_days = total_days % 7;
        let start_weekday = start_date.weekday().number_days_from_monday();

        // Подсчет рабочих дней в неполной неделе
        let additional_working_days = (0..=remaining_days)
            .filter(|&i| (start_weekday + i as u8) % 7 < 5)
            .count() as i64;

        (full_weeks_working_days + additional_working_days) as u16
    }

    pub fn whole_days_since(self, other: AsezDate) -> i64 {
        (other.0 - self.0).whole_days()
    }

    /// Конвертирует дату в `AsezTimestamp` с временем 00:00:00.
    pub fn to_timestamp(self) -> AsezTimestamp {
        AsezTimestamp(self.0.midnight())
    }

    /// Сдвигает дату на `days` дней (через юлианский день).
    pub fn add_days(&self, days: i64) -> Self {
        let j = self.0.julian_day() + days;
        Self(Date::from_julian_day(j))
    }

    /// Конвертация строки с форматом даты БД [`DB_DATE_FORMAT`]
    pub fn try_from_db_format(value: &str) -> Result<Self, String> {
        Date::parse(value, DB_DATE_FORMAT)
            .map(AsezDate)
            .map_err(|_| format!("{} не может быть валидной датой", value))
    }

    /// Конвертация строки с форматом апи [`DATE_FORMAT`]
    pub fn try_from_api_format(value: &str) -> Result<Self, String> {
        Date::parse(value, DATE_FORMAT)
            .map(AsezDate)
            .map_err(|_| format!("{} не может быть валидной датой", value))
    }
}

impl From<time::Date> for AsezDate {
    fn from(date: time::Date) -> Self {
        let sqlx_date = sqlx::types::time::Date::try_from_ymd(
            date.year(),
            date.month().into(),
            date.day(),
        )
        .expect("Валидный time::Date должен произвести валидный sqlx::Date");

        Self(sqlx_date)
    }
}

impl TryFrom<&str> for AsezDate {
    type Error = String;

    /// Пробует разобрать строку как дату: сначала в API-формате (`DD.MM.YYYY`),
    /// затем в формате БД (`YYYY-MM-DD`).
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_api_format(value)
            .or_else(|_| Self::try_from_db_format(value))
            .map_err(|_| format!("{} не может быть валидной датой", value))
    }
}

impl From<AsezDate> for NaiveDate {
    fn from(val: AsezDate) -> NaiveDate {
        NaiveDate::from_ymd_opt(
            val.year(),
            (val.month()).into(),
            (val.day()).into(),
        )
        .expect("Все даты валидны")
    }
}

/// Дефолтная "пустая" дата: `1901-01-01`.
/// Совпадает с `NOT NULL DEFAULT '1901-01-01'` в схеме БД.
impl Default for AsezDate {
    fn default() -> Self {
        AsezDate(Date::try_from_yo(1901, 1).expect("1901.01.01 is valid"))
    }
}
impl<'r> Decode<'r, Postgres> for AsezDate {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        <Date as Decode<'r, Postgres>>::decode(value).map(Self)
    }
}
impl Encode<'_, Postgres> for AsezDate {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        self.0.encode_by_ref(buf)
    }
}
impl Type<Postgres> for AsezDate {
    fn type_info() -> PgTypeInfo {
        Date::type_info()
    }
}
impl PgHasArrayType for AsezDate {
    fn array_type_info() -> PgTypeInfo {
        Date::array_type_info()
    }
}

impl Deref for AsezDate {
    type Target = Date;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Сериализуется как строка `"DD.MM.YYYY"`.
impl Serialize for AsezDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Десериализуется из строки: поддерживает оба формата (API и БД).
impl<'de> Deserialize<'de> for AsezDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        AsezDate::try_from(s.as_str()).map_err(DeError::custom)
    }
}

/// Обёртка над [`sqlx::types::time::PrimitiveDateTime`] с реализацией `Default`.
///
/// Нужна по той же причине, что и [`AsezDate`].
/// Сериализуется как Unix timestamp (секунды), десериализуется из него же --
/// это удобнее для API, чем строки с часовым поясом.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AsezTimestamp(pub PrimitiveDateTime);

impl AsezTimestamp {
    pub fn unix_timestamp(&self) -> i64 {
        self.0.assume_offset(UtcOffset::UTC).unix_timestamp()
    }

    pub fn from_unix_timestamp(timestamp: i64) -> Self {
        let offset = OffsetDateTime::from_unix_timestamp(timestamp);
        AsezTimestamp(PrimitiveDateTime::new(offset.date(), offset.time()))
    }

    pub fn now() -> Self {
        let t = OffsetDateTime::now_utc();
        AsezTimestamp(PrimitiveDateTime::new(t.date(), t.time()))
    }

    /// Текущее время с точностью до микросекунд.
    ///
    /// PostgreSQL хранит `timestamp` с точностью до мкс, поэтому при сравнении
    /// с записанными значениями стандартный `now()` с наносекундами может
    /// давать несоответствие.
    pub fn now_us() -> Self {
        let td = OffsetDateTime::now_utc();
        let t = td.time();
        let t = Time::try_from_hms_micro(
            t.hour(),
            t.minute(),
            t.second(),
            t.microsecond(),
        )
        .expect("ok");
        AsezTimestamp(PrimitiveDateTime::new(td.date(), t))
    }

    /// Возвращает дату по московскому времени (UTC+3).
    pub fn to_moscow_date(&self) -> AsezDate {
        AsezDate(self.0.assume_offset(UtcOffset::hours(3)).date())
    }

    /// Конвертирует временную метку в московский часовой пояс (UTC+3).
    pub fn to_moscow_timestamp(&self) -> Self {
        let t = self.0.assume_utc().to_offset(UtcOffset::hours(3));
        AsezTimestamp(PrimitiveDateTime::new(t.date(), t.time()))
    }

    /// Проверяет, является ли временная метка "пустой".
    ///
    /// В БД могут встречаться две "пустые" даты:
    /// `(0,0,0)` -- настоящий `default` при нулевой инициализации.
    /// `1901-01-01` -- значение, установленное в таблицах БД как `NOT NULL DEFAULT '1901-01-01'`.
    /// Проверяем оба случая, чтобы корректно обработать
    /// "пустые" даты независимо от их источника.
    pub fn is_empty(&self) -> bool {
        let year_month_day = self.0.date().as_ymd();
        year_month_day == (0, 0, 0) || year_month_day == (1901, 1, 1)
    }

    /// Возвращает максимально далёкую дату в будущем (9999-12-31).
    pub fn max() -> Self {
        let offset = OffsetDateTime::from_unix_timestamp(253_402_203_600);
        AsezTimestamp(PrimitiveDateTime::new(offset.date(), offset.time()))
    }

    /// Возвращает дату в часовом поясе UTC+3.
    /// Метод предназначен для отображения данных в Excel,
    /// не подходит для работы с БД.
    pub fn date_offset_3(&self) -> AsezDate {
        AsezDate(self.0.assume_offset(UtcOffset::hours(3)).date())
    }

    /// Форматирует временную метку в строку в часовом поясе UTC+3.
    /// Метод предназначен для отображения данных в Excel,
    /// не подходит для работы с БД.
    pub fn to_text_offset_3(&self) -> String {
        self.0
            .assume_utc()
            .to_offset(UtcOffset::hours(3))
            .format(TIMESTAMP_FORMAT)
    }

    /// Конвертация строки с форматом временной метки БД [`DB_TIMESTAMP_FORMAT`]
    pub fn try_from_db_format(value: &str) -> Result<Self, String> {
        PrimitiveDateTime::parse(value, DB_TIMESTAMP_FORMAT)
            .map(AsezTimestamp)
            .map_err(|_| format!("{} не может быть валидной датой", value))
    }

    /// Конвертация строки с форматом временной метки апи [`TIMESTAMP_FORMAT`]
    pub fn try_from_api_format(value: &str) -> Result<Self, String> {
        PrimitiveDateTime::parse(value, TIMESTAMP_FORMAT)
            .map(AsezTimestamp)
            .map_err(|_| format!("{} не может быть валидной датой", value))
    }
}

impl TryFrom<&str> for AsezTimestamp {
    type Error = String;

    /// Пробует разобрать строку как временную метку: сначала API-формат,
    /// затем формат БД.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_api_format(value)
            .or_else(|_| Self::try_from_db_format(value))
            .map_err(|_| format!("{:?} не может быть валидным таймстемпом", value))
    }
}

impl From<time::PrimitiveDateTime> for AsezTimestamp {
    fn from(value: time::PrimitiveDateTime) -> Self {
        let date = value.date();
        let time = value.time();

        let sqlx_date = sqlx::types::time::Date::try_from_ymd(
            date.year(),
            date.month().into(),
            date.day(),
        )
        .expect("Валидный time::Date должен произвести валидный sqlx::Date");

        let sqlx_time = sqlx::types::time::Time::try_from_hms(
            time.hour(),
            time.minute(),
            time.second(),
        )
        .expect("Валидный time::Time должен произвести валидный sqlx::Time");

        Self(sqlx::types::time::PrimitiveDateTime::new(sqlx_date, sqlx_time))
    }
}

impl From<AsezTimestamp> for NaiveDateTime {
    fn from(val: AsezTimestamp) -> NaiveDateTime {
        DateTime::<Utc>::from_timestamp(val.unix_timestamp(), 0)
            .map(|dt| dt.naive_utc())
            .unwrap_or_default()
    }
}

/// Дефолтный "пустой" timestamp: `1901-01-01 00:00:00`.
impl Default for AsezTimestamp {
    fn default() -> Self {
        let d = Date::try_from_yo(1901, 1).expect("1901.01.01 00:00:00 is valid");
        Self(d.midnight())
    }
}
impl<'r> Decode<'r, Postgres> for AsezTimestamp {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        <PrimitiveDateTime as Decode<'r, Postgres>>::decode(value).map(Self)
    }
}
impl Encode<'_, Postgres> for AsezTimestamp {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        self.0.encode_by_ref(buf)
    }
}
impl Type<Postgres> for AsezTimestamp {
    fn type_info() -> PgTypeInfo {
        PrimitiveDateTime::type_info()
    }
}
impl PgHasArrayType for AsezTimestamp {
    fn array_type_info() -> PgTypeInfo {
        PrimitiveDateTime::array_type_info()
    }
}

impl Display for AsezDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(DATE_FORMAT))
    }
}

impl Display for AsezTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(TIMESTAMP_FORMAT))
    }
}

/// Сериализуется как Unix timestamp (i64) для передачи через API.
impl Serialize for AsezTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.unix_timestamp())
    }
}

/// Десериализуется из Unix timestamp (i64).
impl<'de> Deserialize<'de> for AsezTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let number = i64::deserialize(deserializer)?;
        Ok(AsezTimestamp::from_unix_timestamp(number))
    }
}

impl Deref for AsezTimestamp {
    type Target = PrimitiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod date_tests {
    use sqlx::types::time::Date;

    use crate::asez_timestamp;

    use super::AsezDate;

    #[test]
    fn with_next_weekday() {
        // Суббота, в месяце 31 дня
        let someday = AsezDate(Date::try_from_ymd(2000, 1, 29).unwrap());

        assert_eq!(
            AsezDate(Date::try_from_ymd(2000, 1, 30).unwrap()),
            someday.with_next_weekday(time::Weekday::Sunday)
        );
        assert_eq!(
            AsezDate(Date::try_from_ymd(2000, 2, 1).unwrap()),
            someday.with_next_weekday(time::Weekday::Tuesday)
        );
        assert_eq!(
            AsezDate(Date::try_from_ymd(2000, 2, 5).unwrap()),
            someday.with_next_weekday(time::Weekday::Saturday)
        );
    }

    #[test]
    fn to_moscow_timestamp() {
        let timestamp = asez_timestamp!("08.08.2025 11:14:02");
        let expected = asez_timestamp!("08.08.2025 14:14:02");
        assert_eq!(expected, timestamp.to_moscow_timestamp())
    }
}
