use std::collections::BTreeMap;

use serde::Serialize;

/// Снимок переменных окружения сервиса для диагностического endpoint-а.
///
/// `BTreeMap` (а не `HashMap`) — чтобы переменные в ответе шли в алфавитном порядке,
/// это удобно при ручном просмотре конфига в логах или в Swagger UI.
///
/// По умолчанию (без feature `show_passwords`) значения переменных, заканчивающихся
/// на `PASS` или `PASSWORD`, маскируются звёздочками — чтобы конфиг можно было
/// безопасно вывести в лог или вернуть через debug-endpoint.
#[derive(Debug, Default, Serialize)]
pub struct ServerConfig {
    env_vars: BTreeMap<String, String>,
}

impl ServerConfig {
    const PASSWORD_MASK: &'static str = "*************";

    /// # Описание
    ///
    /// Получение конфигурации сервера
    pub fn new() -> Self {
        let env_vars = std::env::vars()
            .map(|(key, value)| {
                if cfg!(not(feature = "show_passwords")) {
                    Self::mask_password(key, value)
                } else {
                    (key, value)
                }
            })
            .collect();
        Self { env_vars }
    }

    fn mask_password(key: String, value: String) -> (String, String) {
        let key_upper = key.to_uppercase();
        if key_upper.ends_with("PASS") || key_upper.ends_with("PASSWORD") {
            (key, Self::PASSWORD_MASK.to_string())
        } else {
            (key, value)
        }
    }
}
