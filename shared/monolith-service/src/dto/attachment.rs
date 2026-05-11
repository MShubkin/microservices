use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

use super::time::PlanningTimestamp;

/// Обновление структуры иерархии директорий по UUID иерархии
#[derive(Serialize, Default, Debug)]
pub struct UpdateHierarchyReqItem {
    /// Уникальный идентификатор корневого элемента иерархии. Не передается, если иерархия еще не создана и ее необходимо создать.
    pub uuid: Option<Uuid>,
    pub item_list: Vec<Attachment>,
}

/// Обновление структуры иерархии директорий по UUID иерархии
#[derive(Serialize, Default, Debug)]
pub struct UpdateHierarchyReq {
    pub hierarchy_list: Vec<UpdateHierarchyReqItem>,
}

/// Ответ на [обновление структуры иерархии директорий по UUID иерархии](UpdateHierarchyReq)
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UpdateHierarchyResponseData {
    pub hierarchy_list: Vec<UpdateHierarchyResponseItem>,
}

/// Уникальный идентификатор корневого элемента иерархии
/// NB: Монолит может запросто послать и поле `item_list`, но оно
/// нас не интересует.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UpdateHierarchyResponseItem {
    /// Монолит может запросто и не послать uuid, но в этом случае
    // это ошибка.
    pub uuid: Uuid,
}

/// Запрос на получение структуры иерархии директорий по UUID иерархии
#[derive(Serialize, Default, Debug)]
pub struct GetHierarchyReq {
    pub hierarchy_list: Vec<Uuid>,
}

/// Ответ на [получение структуры иерархии директорий по UUID иерархии](GetHierarchyReq)
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GetHierarchyResponseData {
    pub hierarchy_list: Vec<GetHierarchyResponseItem>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GetHierarchyResponseItem {
    pub uuid: Uuid,
    pub item_list: Vec<Attachment>,
}

/// Элемент обновления иерархии (папки или файл)
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(default)] // С монолите ВСЕ поля могут не прийти.
pub struct Attachment {
    /// Уникальный идентификатор элемента иерархии (папки или файла)
    pub uuid: Option<Uuid>,
    pub id: i64,
    /// Наименование директории/категории/или файла, с учетом расширения
    pub text: String,
    /// Признак классификации
    pub is_classified: bool,
    /// Признак удаления
    pub is_removed: bool,
    pub is_archived: bool,
    /// Ссылка на родительский id в рамках иерархии
    pub parent_id: Option<i64>,
    /// Идентификатор категории директории. Связан со справочником Категорий.
    pub category_id: Option<FoldersCategory>,
    /// Тип иерархии. 1-файл, 2-директория. Связан со справочником Типов иерархий
    pub kind_id: i16,
    /// Индекс сортировки элементов одного уровня иерархии
    pub sort_index: Option<i32>,
    pub mime_id: Option<i16>,
    pub size: Option<i64>,
    pub changed_at: Option<PlanningTimestamp>,
    pub changed_by: Option<i32>,
}

/// Запрос на получение структуры иерархии директорий по идентификатору шаблона
#[derive(Serialize, Default, Debug)]
pub struct GetHierarchyTemplateReq {
    pub hierarchy_template_id: i64,
}

/// Ответ на [GetHierarchyTemplateReq]
#[derive(Deserialize, Default, Debug)]
pub struct GetHierarchyTemplateResponseData {
    pub item_list: Vec<GetHierarchyTemplateResponseItem>,
}

#[derive(Deserialize, Default, Debug)]
pub struct GetHierarchyTemplateResponseItem {
    /// Уникальный идентификатор элемента иерархии
    pub id: i64,
    /// Идентификатор категории директории. Связан со справочником Категорий.
    pub category_id: Option<FoldersCategory>,
    /// Наименование директории/категории
    pub text: String,
    /// Индекс сортировки элементов одного уровня иерархии
    pub sort_index: Option<i32>,
    /// Ссылка на родительский id в рамках иерархии
    pub parent_id: Option<i64>,
}

/// Запрос на загрузку одного или нескольких файлов по /rest/folders/v1/upload/file/
/// Содержит список файлов с их содержимым и метаданными
#[derive(Debug)]
pub struct UploadFileReq {
    pub files: Vec<UploadFileItem>,
}

/// Представление файла
#[derive(Debug)]
pub struct UploadFileItem {
    /// Содержимое файла в виде массива байт
    pub bytes: Vec<u8>,
    pub name: String,
    pub r#type: String,
}

/// Ответ на [UploadFileReq]
#[derive(Debug, Deserialize)]
pub struct UploadFileResponse {
    pub status: String,
    pub data: UploadFileResponseItem,
}

/// Данные о загруженных файлах
#[derive(Debug, Deserialize)]
pub struct UploadFileResponseItem {
    pub item_list: Vec<UploadedFileInfo>,
}

/// Информация о загруженном файле, полученная от монолита
#[derive(Debug, Deserialize)]
pub struct UploadedFileInfo {
    pub uuid: Uuid,
    pub text: String,
    pub kind_id: i32,
    pub mime_id: i32,
    pub size: i64,
    pub created_by: i32,
    pub created_at: i64,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, Serialize, Deserialize)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum FoldersCategory {
    TechnicalSpecification = 1,
    ContractDocuments = 2,
    CostCalculation = 3,
    ProposalsFromPotentialParticipants = 4,
    EstimatesProjectDocumentation = 5,
    JustificationOfNeed = 6,
    ATCalculations = 8,
    TenderDocumentation = 9,
    DraftAgreement = 10,
    ContractChangeInformationFormA = 11,
    BasisForAgreementConclusion = 12,
    CalculationsEstimatesComparisonTable = 13,
    ContractAndAgreement = 14,
    BasisForContractConclusion = 15,
    ChairmansOrderForAgreementConclusion = 16,
    TenderDocumentationTemplate = 17,
    AdditionalDocuments = 18,
    TenderDocumentationDuplicate = 19, // Assuming this is a duplicate of 'Документы ТКП'
}

impl FoldersCategory {
    fn as_str(&self) -> &'static str {
        match self {
            FoldersCategory::TechnicalSpecification => "Техническое задание",
            FoldersCategory::ContractDocuments => "Договорные документы",
            FoldersCategory::CostCalculation => "Расчет стоимости",
            FoldersCategory::ProposalsFromPotentialParticipants => {
                "Предложения от потенциальных участников"
            }
            FoldersCategory::EstimatesProjectDocumentation => {
                "Сметы (проектная документация)"
            }
            FoldersCategory::JustificationOfNeed => {
                "Справка-обоснование потребности"
            }
            FoldersCategory::ATCalculations => "Расчеты АЦ",
            FoldersCategory::TenderDocumentation => "Документы ТКП",
            FoldersCategory::DraftAgreement => "Проект ДС",
            FoldersCategory::ContractChangeInformationFormA => {
                "Информация об изменении Договора (форма А)"
            }
            FoldersCategory::BasisForAgreementConclusion => {
                "Документы-основания для заключения ДС"
            }
            FoldersCategory::CalculationsEstimatesComparisonTable => {
                "Расчет, сметы, сравнительная таблица"
            }
            FoldersCategory::ContractAndAgreement => "Договор и ДС",
            FoldersCategory::BasisForContractConclusion => {
                "Документ-основание для заключения Договора"
            }
            FoldersCategory::ChairmansOrderForAgreementConclusion => {
                "Поручение Председателя Правления на заключение ДС"
            }
            FoldersCategory::TenderDocumentationTemplate => "Шаблон ТКП",
            FoldersCategory::AdditionalDocuments => "Дополнительные документы",
            FoldersCategory::TenderDocumentationDuplicate => "Документы ТКП",
        }
    }
}

impl Display for FoldersCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::convert::From<i16> for FoldersCategory {
    fn from(value: i16) -> Self {
        match value {
            1 => FoldersCategory::TechnicalSpecification,
            2 => FoldersCategory::ContractDocuments,
            3 => FoldersCategory::CostCalculation,
            4 => FoldersCategory::ProposalsFromPotentialParticipants,
            5 => FoldersCategory::EstimatesProjectDocumentation,
            6 => FoldersCategory::JustificationOfNeed,
            8 => FoldersCategory::ATCalculations,
            9 => FoldersCategory::TenderDocumentation,
            10 => FoldersCategory::DraftAgreement,
            11 => FoldersCategory::ContractChangeInformationFormA,
            12 => FoldersCategory::BasisForAgreementConclusion,
            13 => FoldersCategory::CalculationsEstimatesComparisonTable,
            14 => FoldersCategory::ContractAndAgreement,
            15 => FoldersCategory::BasisForContractConclusion,
            16 => FoldersCategory::ChairmansOrderForAgreementConclusion,
            17 => FoldersCategory::TenderDocumentationTemplate,
            18 => FoldersCategory::AdditionalDocuments,
            19 => FoldersCategory::TenderDocumentationDuplicate,
            _ => panic!("Invalid value for FoldersCategory: {}", value), // Handle invalid values
        }
    }
}

impl std::convert::From<FoldersCategory> for i16 {
    fn from(category: FoldersCategory) -> i16 {
        category as i16
    }
}
