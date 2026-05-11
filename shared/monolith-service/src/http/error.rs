use crate::dto::dictionary::CommonDictionaryKind;

pub type MonolithHttpResult<T> = std::result::Result<T, MonolithHttpError>;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum MonolithHttpError {
    #[error("Ошибка при настрйоке клиента: {0}")]
    ClientConfig(#[from] reqwest::Error),
    #[error("Невалидный путь: {0}")]
    InvalidPath(#[from] url::ParseError),
    #[error("Попытка совершить недопустимое действие: {0}")]
    Unavailable(String),
    #[error("Ошибка при запросе: {0}")]
    BadRequest(reqwest::Error),
    #[error("Ошибка при получении ответа: {0}")]
    InvalidResponse(reqwest::Error),
    #[error("Справочник {0:?} не был найден")]
    NotFoundDictionary(CommonDictionaryKind),
    #[error("Монолит нарушил правила вернув на запрос {0:?} справочника {1:?} справочник")]
    InvalidDictionary(CommonDictionaryKind, CommonDictionaryKind),
    #[error("Ошибка сериализации JSON: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Ошибка создания части multipart: {0}")]
    Multipart(String),
}
