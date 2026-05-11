use async_trait::async_trait;
use reqwest::multipart::Form;
use serde::{Deserialize, Serialize};

pub mod dto;
pub mod http;

/// Абстракция для обращения к АСЭЗ 1.0
///
/// Для каждого `D` определяется свой перечень
/// апи, с которым можно взаимодействовать. В текущий момент доступно
/// общение с монолитом только с помощью [`MonolithHttpDriver`]
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
