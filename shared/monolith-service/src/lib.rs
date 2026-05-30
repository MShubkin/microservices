use async_trait::async_trait;
use reqwest::multipart::Form;
use serde::{Deserialize, Serialize};

pub mod dto;
pub mod http;

/// Клиент АСЭЗ 1.0 (monolith).
///
/// Параметризован драйвером `D` — это позволяет подменить HTTP-драйвер
/// на mock в тестах без изменения кода бизнес-логики. В продакшне
/// используется единственная реализация `MonolithHttpDriver`.
///
/// Паттерн `MonolithService<D>` + `MonolithDriver` — это "порт и адаптер":
/// `MonolithService` описывает что делать, `D` описывает как.
#[derive(Clone, Debug)]
pub struct MonolithService<D> {
    driver: D,
}

pub use http::MonolithHttpService;

impl<D> MonolithService<D> {
    /// Создание абстракции для обращения к монолиту с помощью
    /// определенного драйвера
    pub fn new(driver: D) -> Self {
        Self { driver }
    }
}

/// Драйвер для обращения к АСЭЗ 1.0
#[async_trait]
pub trait MonolithDriver {
    /// Свойства, которые требуются для обращения
    type Properties;
    /// Ошибка, которая потенциально может возникнуть при использовании драйвера
    type Error;

    /// Общий метод для обращения к монолиту с помощью этого драйвера
    async fn request<B, R>(
        &self,
        body: &B,
        props: Self::Properties,
    ) -> Result<R, Self::Error>
    where
        B: Serialize + Send + Sync,
        R: for<'a> Deserialize<'a>;

    async fn request_blob<B>(
        &self,
        body: &B,
        props: Self::Properties,
    ) -> Result<Vec<u8>, Self::Error>
    where
        B: Serialize + Send + Sync;

    async fn request_multipart<R>(
        &self,
        form: Form,
        props: Self::Properties,
    ) -> Result<R, Self::Error>
    where
        R: for<'a> Deserialize<'a> + Send;
}
