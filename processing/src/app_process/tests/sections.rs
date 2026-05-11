use asez2_shared_db::{db_item::AsezDate, uuid};
use rabbit_services::specialized_departments::SpecializedDepartmentsService;
use shared_essential::presentation::dto::Source;

use super::*;
use crate::app_process;

const SECTIONS_EXTRA_MIGS: &[&str] = &["sections.sql"];

#[tokio::test]
async fn test_get_plans_for_estimated_commission_in_person() {
    run_db_rabbit_test(
        SECTIONS_EXTRA_MIGS,
        |pool, rabbit| async move {
            // Test1: Получение всех элементов, которые подходят под эту
            // секцию
            let request = PlansRequest {
                select: Select::with_fields(["id", "uuid"]),
                section: Section::EstimatedCommissionInPerson,
                user_id: -1,
            };
            // Test 2: Получение одного элемента, который
            // подходит под фильтры секции
            let request2 = PlansRequest {
                select: Select::with_fields(["id", "uuid"]).eq("uuid", 
                    "00000000-0000-0000-0000-000000000001"
                   ),
                section: Section::EstimatedCommissionInPerson,
                user_id: -1,
            };
            // Test 3: Получение одного элемента, который
            // не подходит под фильтры секции
            let request3 = PlansRequest {
                select: Select::with_fields(["id", "uuid"]).eq("uuid", 
                    "00000000-0000-0000-0000-000000000003",),
                section: Section::EstimatedCommissionInPerson,
                user_id: -1,
            };
            // Test 4: Получение элементов для секции +
            // включение элементов protocol_item и agenda_item
            // Один из элементов не должен будет выбраться, так как по элементу Протокола
            // result_id=3, что не подходит
            let request4 = PlansRequest {
                select: Select::with_fields(["id", "agenda_id", "agenda_status_id", "protocol_date", "registration_number", "protocol_id", "protocol_status_id"]),
                section: Section::EstimatedCommissionInPerson,
                user_id: -1,
            };

            let sd_service = SpecializedDepartmentsService::new(rabbit, Default::default(), Source::Processing);

            let r = app_process::get_plans(request, pool.clone(), sd_service.clone()).await.unwrap();
            let r2 = app_process::get_plans(request2, pool.clone(), sd_service.clone()).await.unwrap();
            let r3 = app_process::get_plans(request3, pool.clone(), sd_service.clone()).await.unwrap();
            let r4 = app_process::get_plans(request4, pool.clone(), sd_service.clone()).await.unwrap();

            assert_eq!(r.data.item_list.len(), 3, "Получение всех элементов провалилось: {:#?}", r.data.item_list);
            assert_eq!(r2.data.item_list.len(), 1, "Получение одного элемента провалилось");
            assert_eq!(r3.data.item_list.len(), 0, "Неудачное олучение одного элемента провалилось");
            assert_eq!(
                r4.data.item_list.len(),
                6,
                "Получение всех элементов + элементов с protocol_item ИЛИ agenda_item провалилось"
            );
        },
    )
    .await;
}

/// Конкретно для этой секции логика такова, что независимо от того, запросил ли пользователь
/// поля по Повесткам/Протоколам, буду возвращены все ППЗ/ДС и с Повесткам/Протоколам и без них
#[tokio::test]
async fn test_get_plans_for_estimated_commission_procurements() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Получение всех элементов, которые подходят под эту
        // секцию
        let request = PlansRequest {
            select: Select::with_fields(["id", "uuid"]),
                section: Section::EstimatedCommissionProcurements,
            user_id: -1,
        };
        // Test 2: Получение одного элемента, который
        // подходит под фильтры секции
        let request2 = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000007"),
            section: Section::EstimatedCommissionProcurements,
            user_id: -1,
        };
        // Test 3: Получение одного элемента, который
        // не подходит под фильтры секции
        let request3 = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000009"),
            section: Section::EstimatedCommissionProcurements,
            user_id: -1,
        };
        // Test 4: Получение элементов для секции +
        // включение элементов protocol_item и agenda_item
        let request4 = PlansRequest {
            select: Select::with_fields([
                "id",
                "agenda_id",
                "agenda_status_id",
                "protocol_date",
                "registration_number",
                "protocol_id",
                "protocol_status_id",
            ]),
            section: Section::EstimatedCommissionProcurements,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(rabbit, Default::default(), Source::Processing);

        let r = app_process::get_plans(request, pool.clone(), sd_service.clone()).await.unwrap();
        let r2 = app_process::get_plans(request2, pool.clone(), sd_service.clone()).await.unwrap();
        let r3 = app_process::get_plans(request3, pool.clone(), sd_service.clone()).await.unwrap();
        let r4 = app_process::get_plans(request4, pool.clone(), sd_service.clone()).await.unwrap();

        assert_eq!(
            r.data.item_list.len(),
            19,
            "Получение всех элементов провалилось"
        );
        r.data.item_list.iter().filter_map(|i| i.agenda.as_ref()).for_each(|i| println!("{:?}", i));

        assert_eq!(
            r2.data.item_list.len(),
            1,
            "Получение одного элемента провалилось"
        );

        assert_eq!(
            r3.data.item_list.len(),
            0,
            "Неудачное получение одного элемента провалилось"
        );

        assert_eq!(
            r4.data.item_list.len(),
            19,
            "Получение всех элементов + элементов с protocol_item ИЛИ agenda_item провалилось"
        );
        assert!(r4.data.item_list.iter().any(|i| i.agenda.is_some() || i.protocol.is_some()));
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_for_estimated_commission_no_commission_required() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Получение всех элементов, которые подходят под эту
        // секцию
        let request = PlansRequest {
            select: Select::with_fields(["id", "uuid"]),
            section: Section::EstimatedCommissionNotRequired,
            user_id: -1,
        };
        // Test 2: Получение одного элемента, который
        // подходит под фильтры секции
        let request2 = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000004"),
            section: Section::EstimatedCommissionNotRequired,
            user_id: -1,
        };
        // Test 3: Получение одного элемента, который
        // не подходит под фильтры секции
        let request3 = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000005"),
            section: Section::EstimatedCommissionNotRequired,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let r = app_process::get_plans(request, pool.clone(), sd_service.clone())
            .await
            .unwrap();
        let r2 = app_process::get_plans(request2, pool.clone(), sd_service.clone())
            .await
            .unwrap();
        let r3 = app_process::get_plans(request3, pool.clone(), sd_service.clone())
            .await
            .unwrap();

        assert_eq!(
            r.data.item_list.len(),
            3,
            "Получение всех элементов провалилось"
        );
        assert_eq!(
            r2.data.item_list.len(),
            1,
            "Получение одного элемента провалилось"
        );
        assert_eq!(
            r3.data.item_list.len(),
            0,
            "Неудачное получение одного элемента провалилось"
        );
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_for_estimated_commission_in_correspondence() {
    run_db_rabbit_test(
        SECTIONS_EXTRA_MIGS,
        |pool, rabbit| async move {
            // Test1: Получение всех элементов, которые подходят под эту
            // секцию
            let request = PlansRequest {
                select: Select::with_fields(["id", "uuid"]),
                section: Section::EstimatedCommissionCorrespondence,
                user_id: -1,
            };
            // Test 2: Получение одного элемента, который
            // подходит под фильтры секции
            let request2 = PlansRequest {
                select: Select::with_fields(["id", "uuid"]).eq("uuid","00000000-0000-0000-0000-000000000006"),
                section: Section::EstimatedCommissionCorrespondence,
                user_id: -1,
            };
            // Test 3: Получение одного элемента, который
            // не подходит под фильтры секции
            let request3 = PlansRequest {
                select: Select::with_fields(["id", "uuid"]).eq("uuid", "00000000-0000-0000-0000-000000000008"),
                section: Section::EstimatedCommissionCorrespondence,
                user_id: -1,
            };
            // Test 4: Получение элементов для секции +
            // включение элементов protocol_item
            let request4 = PlansRequest {
                select: Select::with_fields(["id", "protocol_date"]),
                section: Section::EstimatedCommissionCorrespondence,
                user_id: -1,
            };


        let sd_service = SpecializedDepartmentsService::new(rabbit, Default::default(), Source::Processing);

            let r = app_process::get_plans(request, pool.clone(), sd_service.clone()).await.unwrap();
            let r2 = app_process::get_plans(request2, pool.clone(), sd_service.clone()).await.unwrap();
            let r3 = app_process::get_plans(request3, pool.clone(), sd_service.clone()).await.unwrap();
            let r4 = app_process::get_plans(request4, pool.clone(), sd_service.clone()).await.unwrap();

            assert_eq!(r.data.item_list.len(), 4, "Получение всех элементов провалилось");
            assert_eq!(r2.data.item_list.len(), 1, "Получение одного элемента провалилось");
            assert_eq!(r3.data.item_list.len(), 0, "Неудачное получение одного элемента провалилось");
            assert_eq!(
                r4.data.item_list.len(),
                5,
                "Получение всех элементов + элементов с protocol_item и agenda_item провалилось"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_get_plans_for_price_analysis_assing_expert() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Возьмем все планы для `Section::PriceAnalysisAssignExpert`
        let plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"]),
            section: Section::PriceAnalysisAssignExpert,
            user_id: -1,
        };
        // Test 2: Возьмем один план с id для `Section::PriceAnalysisAssignExpert`
        let one_plan_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000001"),
            section: Section::PriceAnalysisAssignExpert,
            user_id: -1,
        };
        // Test 3: Попробуем взять план, что не подходит для `Section::PriceAnalysisAssignExpert`
        let no_plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000003"),
            section: Section::PriceAnalysisAssignExpert,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let one_plan_res =
            app_process::get_plans(one_plan_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let no_plans_res =
            app_process::get_plans(no_plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        assert_eq!(
            plans_res.data.item_list.len(),
            4,
            "Ошибка при получении всех планов"
        );
        assert_eq!(
            one_plan_res.data.item_list.len(),
            1,
            "Ошибка при получении одного плана"
        );
        assert_eq!(
            no_plans_res.data.item_list.len(),
            0,
            "Ошибка при получении плана который нам не нужен"
        );
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_for_price_analysis_determine_price() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Возьмем все планы для `Section::PriceAnalysisDeterminePrice`
        let plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"]),
            section: Section::PriceAnalysisDeterminePrice,
            user_id: -1,
        };
        let one_plan_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000009"),
            section: Section::PriceAnalysisDeterminePrice,
            user_id: -1,
        };
        // Test 3: Попробуем взять план, что не подходит для `Section::PriceAnalysisAssignExpert`
        let no_plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000010"),
            section: Section::PriceAnalysisDeterminePrice,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let one_plan_res =
            app_process::get_plans(one_plan_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let no_plans_res =
            app_process::get_plans(no_plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        assert_eq!(
            plans_res.data.item_list.len(),
            2,
            "Ошибка при получении всех планов"
        );
        assert_eq!(
            one_plan_res.data.item_list.len(),
            1,
            "Ошибка при получении одного плана"
        );
        assert_eq!(
            no_plans_res.data.item_list.len(),
            0,
            "Ошибка при получении плана который нам не нужен"
        );
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_for_price_analysis_approve_price() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Возьмем все планы для `Section::PriceAnalysisApprovePrice`
        let plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"]),
            section: Section::PriceAnalysisApprovePrice,
            user_id: -1,
        };
        // Test 2: Попробуем взять план, что подходит для `Section::PriceAnalysisApprovePrice`
        let one_plan_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000011"),
            section: Section::PriceAnalysisApprovePrice,
            user_id: -1,
        };
        // Test 3: Попробуем взять план, что не подходит для `Section::PriceAnalysisApprovePrice`
        let no_plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000010"),
            section: Section::PriceAnalysisApprovePrice,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let one_plan_res =
            app_process::get_plans(one_plan_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let no_plans_res =
            app_process::get_plans(no_plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        assert_eq!(
            plans_res.data.item_list.len(),
            4,
            "Ошибка при получении всех планов"
        );
        assert_eq!(
            one_plan_res.data.item_list.len(),
            1,
            "Ошибка при получении одного плана"
        );
        assert_eq!(
            no_plans_res.data.item_list.len(),
            0,
            "Ошибка при получении плана который нам не нужен"
        );
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_for_price_analysis_primary_expert_control() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Возьмем все планы для `Section::PriceAnalysisPrimaryExpertControl`
        let plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"]),
            section: Section::PriceAnalysisPrimaryExpertControl,
            user_id: -1,
        };
        // Test 2: Возьмем один план с id для `Section::PriceAnalysisPrimaryExpertControl`
        let one_plan_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0010-000000000000"),
            section: Section::PriceAnalysisPrimaryExpertControl,
            user_id: -1,
        };
        // Test 3: Попробуем взять план, что не подходит для `Section::PriceAnalysisPrimaryExpertControl`
        let no_plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000009"),
            section: Section::PriceAnalysisPrimaryExpertControl,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let one_plan_res =
            app_process::get_plans(one_plan_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let no_plans_res =
            app_process::get_plans(no_plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        assert_eq!(
            plans_res.data.item_list.len(),
            2,
            "Ошибка при получении всех планов"
        );
        assert_eq!(
            one_plan_res.data.item_list.len(),
            1,
            "Ошибка при получении одного плана"
        );
        assert_eq!(
            no_plans_res.data.item_list.len(),
            0,
            "Ошибка при получении плана который нам не нужен"
        );
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_for_price_analysis_gpg() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Возьмем все планы для `Section::PriceAnalysisGgp`
        let plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"]),
            section: Section::PriceAnalysisGgp,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );
        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        assert_eq!(
            plans_res.data.item_list.len(),
            24,
            "Ошибка при получении всех планов"
        );
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_for_working_expert() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Возьмем все планы для `Section::InWorkByExpertDepartment`
        let plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"]),
            section: Section::InWorkByExpertDepartment,
            user_id: 1,
        };
        // Test 2: Возьмем один план с id для `Section::InWorkByExpertDepartment`
        let one_plan_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000001"),
            section: Section::InWorkByExpertDepartment,
            user_id: 1,
        };
        // Test 3: Попробуем взять план, что не подходит для `Section::InWorkByExpertDepartment`
        let no_plan_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000007"),
            section: Section::InWorkByExpertDepartment,
            user_id: 1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let one_plan_res =
            app_process::get_plans(one_plan_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let no_plan_res =
            app_process::get_plans(no_plan_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        assert_eq!(
            plans_res.data.item_list.len(),
            12,
            "Ошибка при получении всех планов"
        );
        assert_eq!(
            one_plan_res.data.item_list.len(),
            1,
            "Ошибка при получении одного плана"
        );
        assert_eq!(
            no_plan_res.data.item_list.len(),
            0,
            "Ошибка при получении одного плана"
        );
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_for_assign_expert_department() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Возьмем все планы для `Section::AssignExpertDepartment`
        let plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"]),
            section: Section::AssignExpertDepartment,
            user_id: -1,
        };
        // Test 2: Возьмем один план с id для `Section::AssignExpertDepartment`
        let one_plan_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000011"),
            section: Section::AssignExpertDepartment,
            user_id: -1,
        };
        // Test 3: Попробуем взять план, что не подходит для `Section::AssignExpertDepartment`
        let no_plan_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .eq("uuid", "00000000-0000-0000-0000-000000000001"),
            section: Section::AssignExpertDepartment,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let one_plan_res =
            app_process::get_plans(one_plan_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let no_plan_res =
            app_process::get_plans(no_plan_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        assert_eq!(
            plans_res.data.item_list.len(),
            5,
            "Ошибка при получении всех планов"
        );
        assert_eq!(
            one_plan_res.data.item_list.len(),
            1,
            "Ошибка при получении одного плана"
        );
        assert_eq!(
            no_plan_res.data.item_list.len(),
            0,
            "Ошибка при получении одного плана"
        );
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_with_plan_sort() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Возьмем все планы для `Section::AssignExpertDepartment`
        let plans_req = PlansRequest {
            select: Select::with_fields(["plan_id", "uuid"])
                .add_replace_order_desc("plan_id"),
            section: Section::AssignExpertDepartment,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        assert_eq!(
            plans_res.data.item_list.len(),
            5,
            "Ошибка при получении всех планов"
        );
        let items = plans_res
            .data
            .item_list
            .into_iter()
            .map(|p| p.plan)
            .collect::<Vec<_>>();
        println!("{:?}", items.iter().map(|x| x.item.id()).collect::<Vec<_>>());
        println!(
            "{:?}",
            items.iter().map(|x| x.item.plan_id()).collect::<Vec<_>>()
        );
        assert!(
            matches!(items[0].item, PlanOrAmendmentRep::Plan(_))
                && items[0].item.plan_id().unwrap() == 12,
            "{:?}",
            items[0].item
        );
        assert!(
            matches!(items[1].item, PlanOrAmendmentRep::Amendment(_))
                && items[1].item.plan_id().unwrap() == 12,
            "{:?}",
            items[1].item
        );
        assert!(
            matches!(items[2].item, PlanOrAmendmentRep::Plan(_))
                && items[2].item.plan_id().unwrap() == 11,
            "{:?}",
            items[2].item
        );
        assert!(
            matches!(items[3].item, PlanOrAmendmentRep::Amendment(_))
                && items[3].item.plan_id().unwrap() == 11,
            "{:?}",
            items[3].item
        );
        assert!(
            matches!(items[4].item, PlanOrAmendmentRep::Plan(_))
                && items[4].item.plan_id().unwrap() == 10,
            "{:?}",
            items[4].item
        );
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_with_plan_sort2() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Возьмем все планы для `Section::AssignExpertDepartment`
        let plans_req = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .add_replace_order_desc("id"),
            section: Section::AssignExpertDepartment,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        assert_eq!(
            plans_res.data.item_list.len(),
            5,
            "Ошибка при получении всех планов"
        );
        let items = plans_res
            .data
            .item_list
            .into_iter()
            .map(|p| p.plan)
            .collect::<Vec<_>>();
        println!("{:?}", items.iter().map(|x| x.item.id()).collect::<Vec<_>>());
        println!(
            "{:?}",
            items.iter().map(|x| x.item.plan_id()).collect::<Vec<_>>()
        );
        assert!(
            matches!(items[0].item, PlanOrAmendmentRep::Plan(_))
                && items[0].item.id().unwrap() == 12,
            "{:?}",
            items[0].item
        );
        assert!(
            matches!(items[1].item, PlanOrAmendmentRep::Amendment(_))
                && items[1].item.id().unwrap() == 12,
            "{:?}",
            items[1].item
        );
        assert!(
            matches!(items[2].item, PlanOrAmendmentRep::Plan(_))
                && items[2].item.id().unwrap() == 11,
            "{:?}",
            items[2].item
        );
        assert!(
            matches!(items[3].item, PlanOrAmendmentRep::Amendment(_))
                && items[3].item.id().unwrap() == 11,
            "{:?}",
            items[3].item
        );
        assert!(
            matches!(items[4].item, PlanOrAmendmentRep::Plan(_))
                && items[4].item.id().unwrap() == 10,
            "{:?}",
            items[4].item
        );
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_agenda_protocol_sort() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        let req = PlansRequest {
            select: Select::with_fields([
                "id",
                "agenda_id",
                "agenda_status_id",
                "protocol_date",
                "registration_number",
                "protocol_id",
                "protocol_status_id",
            ])
            .add_replace_order_desc(EcProtocol::protocol_date),
            section: Section::EstimatedCommissionInPerson,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        assert_eq!(
            plans_res.data.item_list.len(),
            6,
            "Ошибка при получении всех планов"
        );

        let expected_dates = vec![
            AsezDate::try_from("2002-11-11").ok(),
            AsezDate::try_from("2001-11-11").ok(),
            AsezDate::try_from("2000-11-11").ok(),
            None,
            None,
            None,
        ];

        plans_res.data.item_list.into_iter().zip(expected_dates).for_each(
            |(item, date)| {
                assert_eq!(
                    item.protocol
                        .as_ref()
                        .and_then(|protocol| protocol.protocol_date),
                    date,
                    "{:?}",
                    item
                );
            },
        )
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_agenda_protocol_filter() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        let req = PlansRequest {
            select: Select::with_fields([
                "id",
                "uuid",
                "agenda_id",
                "agenda_status_id",
                "protocol_date",
                "registration_number",
                "protocol_id",
                "protocol_status_id",
            ])
            .eq(
                EcProtocol::protocol_date,
                AsezDate::try_from("2000-11-11").unwrap(),
            ),
            section: Section::EstimatedCommissionInPerson,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(req, pool.clone(), sd_service.clone())
                .await
                .unwrap();
        let item_list = plans_res.data.item_list;

        assert_eq!(item_list.len(), 6, "Ошибка при получении всех планов");

        let verify_item = |uuid: &str, has_protocol: bool| {
            let item = item_list
                .iter()
                .find(|i| i.plan.item.uuid().unwrap().to_string() == uuid)
                .unwrap();
            assert_eq!(item.protocol.is_some(), has_protocol, "{:#?}", item);
        };

        verify_item("00000000-0000-0000-0002-000000000000", false);
        verify_item("00000000-0000-0000-0003-000000000000", false);
        verify_item("00000000-0000-0000-0000-000000000003", true);
        verify_item("00000000-0000-0000-0000-000000000001", false);
        verify_item("00000000-0000-0000-0000-000000000002", false);
        verify_item("00000000-0000-0000-0001-000000000000", false);
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_section_mapping() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Вид секций по которой нет маппингов
        let no_mappings = PlansRequest {
            select: Select::with_fields([
                "id",
                "pricing_resume",
                "sum_excluded_vat",
            ])
            .in_any(
                "uuid",
                [
                    "00000000-0000-0000-0000-000000000009",
                    "00000000-0000-0000-0009-000000000000",
                ],
            ),
            section: Section::PriceAnalysisDeterminePrice,
            user_id: -1,
        };
        // Test 2: Вид секций по которой есть маппинги и экстра поля
        let with_mappings = PlansRequest {
            select: Select::with_fields([
                "id",
                "sum_excluded_vat",
                "pricing_resume",
                "protocol_date",
            ])
            .in_any(
                "uuid",
                [
                    "00000000-0000-0000-0000-000000000003",
                    "00000000-0000-0000-0003-000000000000",
                ],
            ),
            section: Section::EstimatedCommissionInPerson,
            user_id: -1,
        };
        // Test 3: Вид секций по которой есть маппинги и нет экстра поля
        let with_mappings_no_extra = PlansRequest {
            select: Select::with_fields([
                "id",
                "sum_excluded_vat",
                "pricing_resume",
            ])
            .in_any(
                "uuid",
                [
                    "00000000-0000-0000-0000-000000000004",
                    "00000000-0000-0000-0004-000000000000",
                ],
            ),
            section: Section::EstimatedCommissionNotRequired,
            user_id: -1,
        };


        let sd_service = SpecializedDepartmentsService::new(rabbit, Default::default(), Source::Processing);

        let plans_res1 = app_process::get_plans(no_mappings, pool.clone(), sd_service.clone()).await.unwrap();
        let plans_res2 =
            app_process::get_plans(with_mappings, pool.clone(), sd_service.clone()).await.unwrap();
        let plans_res3 =
            app_process::get_plans(with_mappings_no_extra, pool.clone(), sd_service.clone()).await.unwrap();

        assert_eq!(plans_res1.data.item_list.len(), 2);
        assert_eq!(plans_res2.data.item_list.len(), 2);
        assert_eq!(plans_res3.data.item_list.len(), 2);

        let verify_plan =
            |plans: &[GetPlansCalculatedItem], uuid: &str, sum_excluded_vat: CurrencyValue| -> bool {
                let plan = plans
                    .iter()
                    .map(|i| &i.plan.item)
                    .find(|p| p.uuid().unwrap() == Uuid::parse_str(uuid).unwrap())
                    .unwrap();

                let check_result = match plan {
                    PlanOrAmendmentRep::Plan(p) => {
                        p.pricing_resume.is_some()
                            && p.sum_excluded_vat.unwrap() == sum_excluded_vat
                    }
                    PlanOrAmendmentRep::Amendment(a) => {
                        a.pricing_resume.is_some()
                            && a.sum_excluded_vat.unwrap() == sum_excluded_vat
                            && a.delta_sum_excluded_vat.is_none()
                    }
                };

                if !check_result {
                    println!("Неверное значение для id={:?} с pricing_resume={:?}, sum_excluded_vat={:?}, delta_sum_excluded_vat={:?}", plan.id(), plan.pricing_resume(), plan.sum_excluded_vat(), match plan {
                        PlanOrAmendmentRep::Amendment(a) => a.delta_sum_excluded_vat,
                        PlanOrAmendmentRep::Plan(_) => None
                    })
                }

                check_result
            };

        assert!(verify_plan(
            &plans_res1.data.item_list,
            "00000000-0000-0000-0000-000000000009",
            1.into()
        ));
        assert!(verify_plan(
            &plans_res1.data.item_list,
            "00000000-0000-0000-0009-000000000000",
            1.into()
        ));

        assert!(verify_plan(
            &plans_res2.data.item_list,
            "00000000-0000-0000-0000-000000000003",
            1.into()
        ));
        assert!(verify_plan(
            &plans_res2.data.item_list,
            "00000000-0000-0000-0003-000000000000",
            2.into()
        ));

        assert!(verify_plan(
            &plans_res3.data.item_list,
            "00000000-0000-0000-0000-000000000004",
            1.into()
        ));
        assert!(verify_plan(
            &plans_res3.data.item_list,
            "00000000-0000-0000-0004-000000000000",
            2.into()
        ));
    })
    .await;
}

/// Тест на работу маппингов в рамках фильтров и сортировок
#[tokio::test]
async fn test_get_plans_section_mapping_filter_ordering() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        let req = PlansRequest {
            select: Select::with_fields([
                ContractAmendment::id,
                ContractAmendment::uuid,
                ContractAmendment::sum_excluded_vat,
            ])
            .in_any(ContractAmendment::sum_excluded_vat, [666, 777])
            .add_replace_order_desc(ContractAmendment::sum_excluded_vat),
            section: Section::EstimatedCommissionCorrespondence,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let res = app_process::get_plans(req, pool.clone(), sd_service.clone())
            .await
            .unwrap();

        // Должно быть именно два элемента
        assert_eq!(res.data.item_list.len(), 2);
        // Порядок должен быть именно таким, так как сортировки идет именно по delta_sum_excluded_vat
        [
            uuid!("00000000-0000-0000-0007-000000000000"),
            uuid!("00000000-0000-0000-0006-000000000000"),
        ]
        .iter()
        .zip(res.data.item_list)
        .for_each(|(&uuid, item)| assert_eq!(item.plan.item.uuid().unwrap(), uuid))
    })
    .await;
}

#[tokio::test]
async fn test_get_plans_tolerated_without_mapping() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        // Test1: Вид секций по которой нет маппингов
        let no_mappings = PlansRequest {
            select: Select::with_fields([
                ContractAmendment::id,
                ContractAmendment::uuid,
                "contract_amendment_sum_excluded_vat",
                "contract_amendment_pricing_sum_excluded_vat",
            ])
            .in_any(
                ContractAmendment::uuid,
                [
                    "00000000-0000-0000-0006-000000000000",
                    "00000000-0000-0000-0007-000000000000",
                ],
            ),
            section: Section::EstimatedCommissionCorrespondence,
            user_id: -1,
        };


        let sd_service = SpecializedDepartmentsService::new(rabbit, Default::default(), Source::Processing);

        let res = app_process::get_plans(no_mappings, pool.clone(), sd_service.clone()).await.unwrap();

        assert_eq!(res.data.item_list.len(), 2);

        let verify_amendment = |plans: &[GetPlansCalculatedItem],
                                uuid: &str,
                                sum_excluded_vat: CurrencyValue,
                                pricing_sum_excluded_vat: CurrencyValue|
         -> bool {
            let plan = plans
                .iter()
                .map(|i| &i.plan.item)
                .find(|p| p.uuid().unwrap() == Uuid::parse_str(uuid).unwrap())
                .unwrap();

            let PlanOrAmendmentRep::Amendment(amendment) = plan else {
                    panic!("{} не является ДС", uuid)
                };

            let check_result = amendment
                .contract_amendment_sum_excluded_vat
                .unwrap()
                == sum_excluded_vat
                && amendment.contract_amendment_pricing_sum_excluded_vat.unwrap()
                    == pricing_sum_excluded_vat
                && amendment.sum_excluded_vat.is_none()
                && amendment.pricing_sum_excluded_vat.is_none()
                && amendment.delta_sum_excluded_vat.is_none()
                && amendment.pricing_delta_sum_excluded_vat.is_none();

            if !check_result {
                println!(
                    "ДС {:?} невалидно с данными contract_amendment_sum_excluded_vat={:?}, contract_amendment_pricing_sum_excluded_vat={:?}, sum_excluded_vat={:?}, pricing_sum_excluded_vat={:?}, delta_sum_excluded_vat={:?}, pricing_delta_sum_excluded_vat={:?}", 
                    amendment.id,
                    amendment.contract_amendment_sum_excluded_vat,
                    amendment.contract_amendment_pricing_sum_excluded_vat,
                    amendment.sum_excluded_vat,
                    amendment.pricing_sum_excluded_vat,
                    amendment.delta_sum_excluded_vat,
                    amendment.pricing_delta_sum_excluded_vat
                );
            }

            check_result
        };

        assert!(verify_amendment(
            &res.data.item_list,
            "00000000-0000-0000-0006-000000000000",
            1.11.into(),
            3.into()
        ));
        assert!(verify_amendment(
            &res.data.item_list,
            "00000000-0000-0000-0007-000000000000",
            2.22.into(),
            3.into()
        ));
    })
    .await;
}

#[tokio::test]
async fn calculated_fields_plans() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        let plans_req = PlansRequest {
            select: Select::with_fields([
                "id",
                "uuid",
                "section_id",
                "pricing_process_count",
                "start_received_date",
                "start_primary_expert_control_date",
                "start_determine_price_date",
                "start_approved_date",
                "pricing_working_days_count_threshold",
                "number_of_days_with_expert_threshold",
            ]),
            section: Section::PriceAnalysisApprovePrice,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        let all_plans_valid = plans_res.data.item_list.iter().any(|plan| {
            plan.plan.calculated.pricing_process_count.is_some()
                || plan.plan.calculated.start_received_date.is_some()
                || plan.plan.calculated.start_primary_expert_control_date.is_some()
                || plan.plan.calculated.start_determine_price_date.is_some()
                // Сейчас для теста выполнено для всех ППЗ/ДС
                // только условие по pricing_working_days_count_threshold
                // По всем не больше 6 дней, так как везде pricing_started_at = now() - 7 days
                // Текущий день тоже берется в учет
                && plan
                    .plan
                    .calculated
                    .pricing_working_days_count_threshold
                    .unwrap().value <= 6
        });

        assert!(
            all_plans_valid,
            "Ни один из планов не содержит ни одного требуемого поля!"
        );
    })
    .await;
}

#[tokio::test]
async fn calculated_fields_protocol_item() {
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        let plans_req = PlansRequest {
            select: Select::with_fields([
                "id",
                "uuid",
                "commission_sum_excluded_vat",
                "commission_percent_economy",
                "commission_economy_sum_excluded_vat",
                "vote_iteraction_price",
            ])
            .add_replace_order_desc("commission_sum_excluded_vat"),
            section: Section::EstimatedCommissionInPerson,
            user_id: -1,
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let plans_res =
            app_process::get_plans(plans_req, pool.clone(), sd_service.clone())
                .await
                .unwrap();

        let expected_values = vec![
            (CurrencyValue::from(3), String::from("25,00"), CurrencyValue::from(1)),
            (CurrencyValue::from(2), String::from("33,33"), CurrencyValue::from(1)),
            (CurrencyValue::from(1), String::from("50,00"), CurrencyValue::from(1)),
        ];
        plans_res
            .data
            .item_list
            .iter()
            .filter(|i| i.protocol_item.is_some())
            .zip(expected_values)
            .for_each(|(item, expected_val)| {
                let protocol_item = item.protocol_item.as_ref().unwrap();
                let val = (
                    protocol_item
                        .item
                        .commission_sum_excluded_vat
                        .unwrap()
                        .unwrap(),
                    protocol_item
                        .calculated
                        .commission_percent_economy
                        .clone()
                        .unwrap(),
                    protocol_item
                        .calculated
                        .commission_economy_sum_excluded_vat
                        .unwrap(),
                );
                let check = expected_val == val;

                assert!(check, "{:?} IN {:?}", expected_val, protocol_item);
            });
    })
    .await;
}

#[tokio::test]
async fn get_plans_paginated() {
    const LIMIT: usize = 5;
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        let request = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .add_replace_order_desc("id"),
            section: Section::EstimatedCommissionProcurements,
            user_id: -1,
        };

        let mut paginated_request = request.clone();
        paginated_request.select =
            paginated_request.select.take_n(LIMIT).offset(0).count_total(true);

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let r = app_process::get_plans(
            request.clone(),
            pool.clone(),
            sd_service.clone(),
        )
        .await
        .unwrap();
        let all_data = r.data.item_list;

        let mut paginated_data = vec![];
        loop {
            let r = app_process::get_plans(
                paginated_request.clone(),
                pool.clone(),
                sd_service.clone(),
            )
            .await
            .unwrap();
            if paginated_request.select.count_total == Some(true) {
                assert_eq!(r.data.total, Some(all_data.len()));
            }
            paginated_request.select.count_total = None;
            let len = r.data.item_list.len();
            *paginated_request.select.offset.as_mut().unwrap() += len;
            paginated_data.extend(r.data.item_list);

            if len == 0 {
                break;
            }

            if paginated_data.len() != all_data.len() {
                assert_eq!(len, LIMIT);
            } else {
                assert!(len <= LIMIT);
            }
        }

        assert_eq!(all_data, paginated_data);
    })
    .await;
}

#[tokio::test]
async fn get_plans_filtered_paginated() {
    const LIMIT: usize = 5;
    run_db_rabbit_test(SECTIONS_EXTRA_MIGS, |pool, rabbit| async move {
        let request = PlansRequest {
            select: Select::with_fields(["id", "uuid"])
                .in_any("status_id", [221, 222, 223])
                .add_replace_order_desc("id"),
            section: Section::EstimatedCommissionProcurements,
            user_id: -1,
        };

        let mut paginated_request = request.clone();
        paginated_request.select =
            paginated_request.select.take_n(LIMIT).offset(0).count_total(true);

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );

        let r = app_process::get_plans(
            request.clone(),
            pool.clone(),
            sd_service.clone(),
        )
        .await
        .unwrap();
        let all_data = r.data.item_list;

        let mut paginated_data = vec![];
        loop {
            let r = app_process::get_plans(
                paginated_request.clone(),
                pool.clone(),
                sd_service.clone(),
            )
            .await
            .unwrap();
            if paginated_request.select.count_total == Some(true) {
                assert_eq!(r.data.total, Some(all_data.len()));
            }
            paginated_request.select.count_total = None;
            let len = r.data.item_list.len();
            *paginated_request.select.offset.as_mut().unwrap() += len;
            paginated_data.extend(r.data.item_list);

            if len == 0 {
                break;
            }

            if paginated_data.len() != all_data.len() {
                assert_eq!(len, LIMIT);
            } else {
                assert!(len <= LIMIT);
            }
        }

        assert_eq!(all_data, paginated_data);
    })
    .await;
}
