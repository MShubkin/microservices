use serde::{Deserialize, Serialize};
/// Запрос на получение количества прикреплённых документов к ППЗ/ДС
#[derive(Serialize, Default, Debug)]
pub struct GetFilesCountRequest {
    pub item_list: Vec<String>,
}

/// Ответ на запрос на получение количества прикреплённых документов к ППЗ/ДС
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GetFilesCountResponse {
    pub value: Vec<FilesCount>,
}
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct FilesCount {
    pub id: String,
    pub count: i16,
}
