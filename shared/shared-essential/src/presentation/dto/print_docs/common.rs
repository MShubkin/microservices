use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TemplateFormat {
    Csv,
    Docx,
    Pdf,
    #[default]
    Xlsx,
}

impl TemplateFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        let lowercased = ext.trim().to_lowercase();
        if lowercased.ends_with("xsl") || lowercased.ends_with("xslx") {
            Some(TemplateFormat::Xlsx)
        } else if lowercased.ends_with("doc") || lowercased.ends_with("docx") {
            Some(TemplateFormat::Docx)
        } else if lowercased.ends_with("csv") || lowercased.ends_with("tsv") {
            Some(TemplateFormat::Csv)
        } else if lowercased.ends_with("pdf") {
            Some(TemplateFormat::Pdf)
        } else {
            None
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            TemplateFormat::Csv => "csv",
            TemplateFormat::Docx => "docx",
            TemplateFormat::Pdf => "pdf",
            TemplateFormat::Xlsx => "xlsx",
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            TemplateFormat::Csv => "application/csv",
            TemplateFormat::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            TemplateFormat::Pdf => "application/pdf",
            TemplateFormat::Xlsx => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct TemplateStructure {
    /// Индекс строки, содержащей наименования столбцов
    pub column_names_row_index: i16,
    /// Индекс строки, содержащей идентификаторы столбцов
    pub column_ids_row_index: Option<i16>,
    /// Индекс строки, с которого начинаются данные
    pub data_start_row_index: i16,
}
