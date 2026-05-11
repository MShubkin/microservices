//! Общие задачи при работе со Сметной Комиссией
use asez2_shared_db::db_item::AsezDate;
use time::{macros::offset, Duration, OffsetDateTime, Weekday};

pub fn is_commission_date_possible(commission_date: AsezDate) -> bool {
    // Текущее время Московское
    let now_moscow = OffsetDateTime::now_utc().to_offset(offset!(+3));

    inner_commission_date_check(now_moscow, commission_date)
}

// Разделим на две функции для более удобного тестирования
#[inline]
fn inner_commission_date_check(
    now_moscow: OffsetDateTime,
    commission_date: AsezDate,
) -> bool {
    // Если текущее время < 13:00 и текущий день = понедельник, то можно выбирать любую дату
    if now_moscow.hour() < 13 && now_moscow.date().weekday() == Weekday::Monday {
        return true;
    }

    // Количество дней с начала этой недели (понедельника)
    let number_days_from_monday =
        now_moscow.weekday().number_days_from_monday() as i64;

    // Первая разрешенная дата
    let enabled_date =
        now_moscow.date() + Duration::days(7 - number_days_from_monday);

    // Если дата комиссии больше или равна первой разрешенной даты, то выбрать ее можно
    commission_date
        >= AsezDate::from_julian_day(enabled_date.to_julian_day() as i64)
}

#[cfg(test)]
mod tests {
    use time::{Duration, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

    use super::*;

    #[test]
    fn test_commission_date_before_monday() {
        // Текущая дата до 12:00 понедельника
        let time_at_1159 = Time::from_hms(11, 59, 0).unwrap();
        let current_monday_at_11_59 = current_monday_at_time(time_at_1159);

        let commission_date = AsezDate::from_julian_day(
            OffsetDateTime::now_utc().date().to_julian_day() as i64,
        );
        assert!(inner_commission_date_check(
            current_monday_at_11_59,
            commission_date
        ));
    }

    #[test]
    fn test_commission_date_after_monday() {
        // Текущая дата после 12:00 понедельника
        let time_at_1201 = Time::from_hms(12, 1, 0).unwrap();
        let current_monday_at_12_01 = current_monday_at_time(time_at_1201);
        let commission_date = AsezDate::from_julian_day(
            OffsetDateTime::now_utc().date().to_julian_day() as i64,
        );

        assert!(!inner_commission_date_check(
            current_monday_at_12_01,
            commission_date
        ));
    }

    fn current_monday_at_time(time: Time) -> OffsetDateTime {
        // Смещение для московского времени (UTC+3)
        let moscow_offset = UtcOffset::from_hms(3, 0, 0).unwrap();

        // Текущее московское время
        let now_moscow = OffsetDateTime::now_utc().to_offset(moscow_offset);

        // Вычисляем количество дней, прошедших с понедельника
        let days_since_monday =
            now_moscow.weekday().number_days_from_monday() as i64;
        // Дата текущего понедельника
        let current_monday_date =
            now_moscow.date() - Duration::days(days_since_monday);

        PrimitiveDateTime::new(current_monday_date, time)
            .assume_offset(moscow_offset)
    }
}
