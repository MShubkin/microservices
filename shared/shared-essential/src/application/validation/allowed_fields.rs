use std::collections::HashSet;
use thiserror::Error;

/// # Описание
///
/// Валидация доступных пользователю полей
///
/// # Аргументы
/// * `selected_fields` - Выбранные пользователем поля
/// * `allowed_fields` - Разрешенные поля
///
/// # Возвращает
/// * None - Все поля разрешены для получения
/// * Some(Vec<String>) - Массив неразрешенных полей
pub fn validate_allowed_fields<D: std::fmt::Display>(
    selected_fields: &[String],
    allowed_fields: &[String],
    user_id: D,
) -> Result<(), ForbiddenFieldError> {
    let allowed_fields_set: HashSet<_> = allowed_fields.iter().collect();
    let selected_fields_set: HashSet<_> = selected_fields.iter().collect();

    let not_allowed_fields = selected_fields_set
        .difference(&allowed_fields_set)
        .map(|field| (*field).clone())
        .collect::<Vec<_>>();

    if not_allowed_fields.is_empty() {
        Ok(())
    } else {
        tracing::warn!(
            kind = "siem",
            "[SIEM] Пользователь {} пытался получить недоступные ему поля: {:?}",
            user_id,
            not_allowed_fields
        );
        Err(ForbiddenFieldError(not_allowed_fields))
    }
}

/// Ошибка проверки полей.
#[derive(Debug, Error)]
#[error("У Пользователя отсутствует авторизация на следующие поля: {0:?}")]
pub struct ForbiddenFieldError(pub Vec<String>);
