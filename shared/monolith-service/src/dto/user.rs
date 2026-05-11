use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct MonolithUser {
    pub id: i32,
    pub email: String,
    pub sap_code: String,
    pub person_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub patronymic_name: String,
    /// Полное имя пользователя: first_name + last_name + patronymic_name
    pub text: String,
    pub phone: String,
    pub unit_id: i32,
    pub is_removed: Option<bool>,
    pub changed_by: Option<i32>,
    pub changed_at: Option<i64>,
}

impl MonolithUser {
    pub fn fio(&self) -> String {
        match self.patronymic_name.is_empty() {
            true => format!("{} {}", self.last_name, self.first_name),
            false => format!(
                "{} {} {}",
                self.last_name, self.first_name, self.patronymic_name
            ),
        }
    }

    /// Имя в формате `Фамилия И.О`
    pub fn ui_text(&self) -> String {
        let first_name_c = self.first_name.chars().next().unwrap_or('?');
        let patronyc_name_c = self.patronymic_name.chars().next().unwrap_or('?');
        let last_name = &self.last_name;

        format!("{last_name} {first_name_c}.{patronyc_name_c}.")
    }
}
