use crate::master_data::price_analysis::conclusion_template_variables::ConclusionTemplateVariables;
use crate::master_data::price_analysis::sample_conclusion::SampleConclusion;
use crate::master_data::price_analysis::sample_conclusion_crit::SampleConclusionCrit;
use serde::{Deserialize, Serialize};

/// Объединяющая структура, в которую входит
/// один шаблон и много критериев + много переменных
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct ConclusionTemplateDictionary {
    /// Шаблон
    pub template: SampleConclusion,
    /// Критерии
    pub crits: Vec<SampleConclusionCrit>,
    /// Переменные
    pub vars: Vec<ConclusionTemplateVariables>,
}
