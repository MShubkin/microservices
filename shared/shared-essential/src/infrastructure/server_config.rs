use std::collections::BTreeMap;

use serde::Serialize;

/// Конфигурация сервера
#[derive(Debug, Default, Serialize)]
pub struct ServerConfig {
    /// Переменные среды
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
