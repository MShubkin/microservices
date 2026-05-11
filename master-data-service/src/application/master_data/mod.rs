pub mod base;
pub mod favorites;
pub mod org_user_assignment_search;
pub mod organizational_structure;

use std::sync::Arc;

use tokio::sync::OnceCell;
use tracing::info;

use shared_essential::{
    domain::master_data::*,
    presentation::dto::master_data::error::{MasterDataError, MasterDataResult},
};

pub use crate::application::master_data::base::MasterDataCommonDirectory;
use crate::{domain::MasterDataDirectoryInterface, infrastructure::GlobalConfig};

use base::{
    budget_item::BudgetItemDirectory, category::CategoryDirectory,
    okpd2::Okpd2Directory,
};

pub(crate) static MASTER_DATA: OnceCell<MasterData> = OnceCell::const_new();
pub fn get_master_data() -> MasterDataResult<&'static MasterData> {
    MASTER_DATA.get().ok_or_else(|| {
        MasterDataError::InternalError("Master Data not loaded".to_string())
    })
}

pub async fn load_master_data(config: Arc<GlobalConfig>) -> MasterDataResult<()> {
    let mut data = MasterData::default();
    data.load_in_memory(config).await?;
    MASTER_DATA.set(data).map_err(|error| {
        MasterDataError::InternalError(format!(
            "Ошибка инициализации переменной MASTER_DATA: {}",
            error
        ))
    })?;
    Ok(())
}

macro_rules! master_data {
    ($md:ident { $($(#[$meta:meta])* $name:ident: $ty:ty),* $(,)? }) => {
        #[derive(Default)]
        pub struct $md {
            $(
                $(#[$meta])*
                pub(crate) $name: $ty,
            )*
        }

        impl $md {
            pub(crate) async fn load_in_memory(
                &mut self,
                config: Arc<GlobalConfig>,
            ) -> MasterDataResult<()> {
                $(
                    self.$name.load(&config.db_pool).await?;
                    info!(kind = "master_data", stringify!("load ", $name));
                )*

                Ok(())
            }
        }

    };
}

master_data! {
    MasterData {
        /// Справочник "Избранных справочников"
        favorite_dictionaries:
            MasterDataCommonDirectory<FavoriteDictionary>,
        /// Справочник "Статья бюджета".
        budget_item: BudgetItemDirectory,
        /// Справочник "ВПЗ"
        category: CategoryDirectory,
        /// Цветовые схемы критичности
        critical_type_color_scheme:
            MasterDataCommonDirectory<CriticalTypeColorScheme>,
        /// Типы заключений эксперта
        expert_conclusion_type:
            MasterDataCommonDirectory<ExpertConclusionType>,
        /// Типы объектов
        object_type: MasterDataCommonDirectory<ObjectType>,
        /// Поставщики
        organization: MasterDataCommonDirectory<Organization>,
        /// Статусы повестки
        agenda_status:
            MasterDataCommonDirectory<EstimatedCommissionAgendaStatus>,
        /// Статусы протокола
        estimated_commission_protocol_status:
            MasterDataCommonDirectory<EstimatedCommissionProtocolStatus>,
        /// Типы протокола
        protocol_type:
            MasterDataCommonDirectory<EstimatedCommissionProtocolType>,
        /// Решения комисии СК по ППЗ/ДС
        estimated_commission_result:
            MasterDataCommonDirectory<EstimatedCommissionResult>,
        /// Роли пользователей Сметной комиссии
        estimated_commission_role:
            MasterDataCommonDirectory<EstimatedCommissionRole>,
        /// Типы Запросов ЗЦИ
        price_information_request_type:
            MasterDataCommonDirectory<PriceInformationRequestType>,
        /// Справочник "Производственный календарь"
        scheduler_request_update_catalog:
            MasterDataCommonDirectory<SchedulerRequestUpdateCatalog>,
        /// Справочник "Способ назначения исполнителя"
        assigning_executor_method:
            MasterDataCommonDirectory<AssigningExecutorMethod>,
        /// Справочник "Способ анализа"
        analysis_method: MasterDataCommonDirectory<AnalysisMethod>,
        /// Справочник "Метод ценообразования"
        price_analysis_method:
            MasterDataCommonDirectory<PriceAnalysisMethod>,
        /// Справочник «Департамент (организация) АЦ»
        pricing_organization_unit: MasterDataCommonDirectory<PricingUnit>,
        /// Справочник "Условия оплаты"
        payment_conditions: MasterDataCommonDirectory<PaymentCondition>,
        /// Справочник "Тип ППЗ"
        ppz_type: MasterDataCommonDirectory<PpzType>,
        response: MasterDataCommonDirectory<Response>,
        /// Справочник «Тип вложенного документа»
        attachment_type: MasterDataCommonDirectory<AttachmentType>,
        /// Справочник "Выходных форм"
        output_form: MasterDataCommonDirectory<OutputForm>,
        /// Справочник "Статусы ТКП"
        technical_commercial_proposal_status:
            MasterDataCommonDirectory<TcpStatus>,
        /// Справочник "Коды ОКПД2".
        okpd2: Okpd2Directory,
        /// Организационная структура.
        organizational_structure: MasterDataCommonDirectory<OrganizationalStructure>,
        /// Справочник "Основания аннулирования"
        plan_reasons_cancel_impact_area: MasterDataCommonDirectory<PlanReasonCancelImpactArea>,
        /// Справочник "Функциональность"
        plan_reasons_cancel_functionality: MasterDataCommonDirectory<PlanReasonCancelFunctionality>,
        /// Справочник "Проверки для ППЗ"
        plan_reasons_cancel_check_reason: MasterDataCommonDirectory<PlanReasonCancelCheckReason>,
    }
}
