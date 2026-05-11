use std::fmt::Display;
use std::time::Duration;

use env_setup::TimeWithOffset;
use futures::Future;
use humantime::format_duration;
use time::{OffsetDateTime, Time};
use tokio::time::sleep;

/// Структура фоновой задачи
pub struct DailyTask {
    name: &'static str,
    time: TimeWithOffset,
}

impl DailyTask {
    pub fn new(name: &'static str, time: TimeWithOffset) -> Self {
        Self { name, time }
    }
}

/// Основной цикл задачи
pub fn spawn_daily_task<Fun, F, E>(task: DailyTask, run: Fun)
where
    Fun: Fn() -> F + Send + Sync + 'static,
    F: Future<Output = Result<(), E>> + Send + 'static,
    E: Display,
{
    tokio::spawn(async move {
        let target_time = task.time.time;
        let target_offset = task.time.offset;

        loop {
            let sleep_duration = duration_within_24h(
                OffsetDateTime::now_utc().to_offset(target_offset).time(),
                target_time,
            );

            tracing::info!(
                "Задача {} запланирована через {}",
                task.name,
                format_duration(sleep_duration)
            );

            sleep(sleep_duration).await;

            tracing::info!("Выполнение задачи: {}", task.name);

            if let Err(error) = run().await {
                tracing::info!("Ошибка запуска задачи: {} {}", task.name, error);
            }
        }
    });
}

/// Вычисляет продолжительность между двумя моментами времени,
/// учитывая возможность перехода через полночь.
fn duration_within_24h(from: Time, to: Time) -> Duration {
    let sleep_duration = if from < to {
        to - from
    } else {
        Duration::from_secs(60 * 60 * 24) - (from - to)
    };
    Duration::from_nanos(
        sleep_duration
            .whole_nanoseconds()
            .try_into()
            .expect("Время сна всегда положительно и меньше 1 дня"),
    )
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };
    use tokio::time::sleep;

    use env_setup::TimeWithOffset;
    use time::{OffsetDateTime, Time, UtcOffset};

    use super::{spawn_daily_task, DailyTask};

    #[tokio::test]
    async fn duration_within_24h() {
        [
            (
                Time::from_hms(10, 30, 0),
                Time::from_hms(12, 30, 0),
                Duration::from_secs(60 * 60 * 2),
            ),
            (
                Time::from_hms(10, 30, 0),
                Time::from_hms(9, 30, 0),
                Duration::from_secs(60 * 60 * 23),
            ),
        ]
        .into_iter()
        .for_each(|(from, to, exp)| {
            assert_eq!(super::duration_within_24h(from.unwrap(), to.unwrap()), exp);
        })
    }

    #[tokio::test]
    async fn one_run_per_day() {
        let counter = Arc::new(AtomicUsize::new(0));
        let now = OffsetDateTime::now_utc().time();
        let sleepy_eepy = Duration::from_secs(1);

        let counter_clone = counter.clone();
        spawn_daily_task(
            DailyTask::new(
                "one_run_per_day",
                TimeWithOffset {
                    offset: UtcOffset::UTC,
                    time: now + sleepy_eepy,
                },
            ),
            move || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Result::<(), String>::Ok(())
                }
            },
        );

        sleep(sleepy_eepy * 2).await;

        let count = counter.load(Ordering::SeqCst);
        assert_eq!(count, 1);
    }
}
