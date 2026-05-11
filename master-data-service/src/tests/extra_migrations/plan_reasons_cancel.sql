-- Фикстуры для тестов справочника "Причины аннулирования"

INSERT INTO public.plan_reason_cancel (id, text, impact_area_id, is_objective_reason, is_new_plan, is_removed, is_reason_fill_type, functionality_id_list, check_reason_id, created_at, created_by, changed_at, changed_by) VALUES
    (1, 'Initial Test Reason', 1, true, false, false, false, '{1}', 1, now(), 1, now(), 1), 
    (2, 'Another Specific Reason Word', 1, false, true, false, true, '{2}', 2, now(), 1, now(), 1);

INSERT INTO public.plan_reason_customer (plan_reason_cancel_id, customer_id, is_removed, created_at, created_by, changed_at, changed_by) VALUES
    (1, 101, false, now(), 1, now(), 1);

SELECT setval('public.plan_reason_cancel_id_seq', (SELECT GREATEST(MAX(id), 2) FROM public.plan_reason_cancel), true);
SELECT setval('public.plan_reason_customer_id_seq', (SELECT GREATEST(MAX(id), 1) FROM public.plan_reason_customer), true);