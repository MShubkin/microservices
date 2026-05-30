use std::fmt::Display;
use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use tokio::time::sleep;
#[cfg(feature = "traces")]
use tracing::warn;

/// Параметры механизма ретрая при работе с брокером.
///
/// Операция выполняется суммарно `attempts + 1` раз: `attempts` повторных попыток
/// плюс одна финальная без перехвата ошибки (см. реализацию [`retry`]).
///
/// # Значения по умолчанию
/// * `attempts` - 5
/// * `sleep_interval_ms` - 200 миллисекунд
#[derive(Deserialize, Clone, Debug)]
pub struct RetryArgs {
    /// Сколько раз повторить операцию после первой неудачи.
    attempts: u32,
    /// Пауза между попытками — достаточно большая для кратковременного сбоя сети,
    /// но не настолько большая, чтобы заметно тормозить старт сервиса.
    sleep_interval_ms: Duration,
}

impl RetryArgs {
    pub fn new(attempts: u32, sleep_interval_ms: u64) -> Self {
        RetryArgs {
            attempts,
            sleep_interval_ms: Duration::from_millis(sleep_interval_ms),
        }
    }
}

impl Default for RetryArgs {
    fn default() -> Self {
        RetryArgs::new(5, 200)
    }
}

/// Выполняет `operation` повторно при неудаче, согласно [`RetryArgs`].
///
/// Цикл идёт `attempts` итераций: при ошибке — пауза и следующая попытка.
/// После цикла идёт финальный вызов без обёртки: его `Err` возвращается напрямую.
/// Итого операция вызывается максимум `attempts + 1` раз.
///
/// `operation` принимается как `Fn` (а не `FnOnce`), чтобы замыкание можно
/// было вызвать несколько раз — каждый вызов создаёт новый `Future`.
///
/// # Аргументы
/// * `operation` - Функция, которая возвращает [`Future`] c [`Future::Output`]=[`Result<T, E>`]
/// * `retry_args` - [Аргументы](RetryArgs) ретрай механизма
///
/// # Возвращает
/// * Ok([`T`]) - Успешно выполненная операция
/// * Err([`E`]) - Операция исчерпала лимит попыток
pub async fn retry<Op, F, T, E>(
    operation: Op,
    retry_args: &RetryArgs,
) -> Result<T, E>
where
    Op: Fn() -> F,
    F: Future<Output = Result<T, E>>,
    E: Display,
{
    let attempts = retry_args.attempts;
    let sleep_interval_ms = retry_args.sleep_interval_ms;

    for attempt in 0..attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(_err) => {
                #[cfg(feature = "traces")]
                warn!(
                    kind = "broker",
                    "Попытка перевыполнить операцию, попытка = {}, пауза = {} ms: {}",
                    attempt,
                    sleep_interval_ms.as_millis(),
                    _err
                );
                // Без feature `traces` `attempt` неиспользуется; явное `_` подавляет warning,
                // не мешая чтению переменной выше.
                #[cfg(not(feature = "traces"))]
                let _ = attempt;
                sleep(sleep_interval_ms).await;
            }
        }
    }

    // Последняя попытка: её результат отдаётся вызывающему коду как есть.
    operation().await
}

/// Трейт-расширение для замыканий: позволяет вызывать `.retry(args)` прямо на них,
/// не передавая функцию первым аргументом. Это удобнее при работе с методами объектов —
/// замыкание захватывает `&self` сервиса и передаётся как `op`.
#[async_trait]
pub trait Retryable<T, E> {
    /// Повторяет вызов `self` согласно [`RetryArgs`].
    /// Поведение идентично [`retry`]: `attempts + 1` вызовов максимум.
    ///
    /// # Аргументы
    /// * `retry_args` - [Аргументы](RetryArgs) ретрай механизма
    ///
    /// # Возвращает
    /// * Ok([`T`]) - Успешно выполненная операция
    /// * Err([`E`]) - Операция исчерпала лимит попыток
    async fn retry(&self, retry_args: &RetryArgs) -> Result<T, E>;
}

/// Бланкетная реализация для любого `Fn() -> Future` — не нужно писать адаптер
/// под каждый конкретный вызов. Ограничения `Send + Sync` обязательны из-за `async_trait`.
#[async_trait]
impl<Op, F, T, E> Retryable<T, E> for Op
where
    F: Future<Output = Result<T, E>> + Send,
    T: Send,
    E: Display + Send,
    Op: Fn() -> F + Send + Sync,
{
    async fn retry(&self, retry_args: &RetryArgs) -> Result<T, E> {
        retry(self, retry_args).await
    }
}

#[cfg(test)]
mod retry_tests {
    use std::io::Error;
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::{RetryArgs, Retryable};

    /// Тестовый стейт
    struct State {
        count: AtomicU32,
    }

    impl State {
        /// Увеличивает счетчик стейта до `until`
        async fn increase_until(
            &self,
            until: u32,
            retry_args: &RetryArgs,
        ) -> Result<(), Error> {
            let op = || async {
                let count = self.count.load(Ordering::Relaxed);

                if count == until {
                    Ok(())
                } else {
                    self.count.store(count + 1, Ordering::Relaxed);
                    Err(Error::last_os_error())
                }
            };
            op.retry(retry_args).await
        }
    }

    /// Тестирует кейс на удачный ретрай в рамках количества попыток
    #[tokio::test]
    async fn success_retry() {
        let attempts = 5;
        let retry_args = RetryArgs::new(attempts, 200);
        let state = State {
            count: AtomicU32::new(0),
        };

        let res = state.increase_until(attempts, &retry_args).await;
        assert!(res.is_ok());
        assert_eq!(state.count.load(Ordering::Relaxed), attempts);
    }

    /// Тестирует кейс при 0 количестве попыток, то есть попыток выполнить операцию будет 1
    #[tokio::test]
    async fn zero_retries() {
        let attempts = 0;
        let retry_args = RetryArgs::new(attempts, 200);
        let state = State {
            count: AtomicU32::new(0),
        };

        let res = state.increase_until(0, &retry_args).await;
        assert!(res.is_ok());
        assert_eq!(state.count.load(Ordering::Relaxed), 0);
    }

    /// Тестирует кейс на неудачны ретрай и выход за рамки количества попыток
    #[tokio::test]
    async fn unreachable_ensure() {
        let attempts = 5;
        let retry_args = RetryArgs::new(attempts, 200);
        let state = State {
            count: AtomicU32::new(0),
        };

        let res = state.increase_until(attempts * 2, &retry_args).await;
        assert!(res.is_err());
        assert_eq!(state.count.load(Ordering::Relaxed), attempts + 1);
    }
}
