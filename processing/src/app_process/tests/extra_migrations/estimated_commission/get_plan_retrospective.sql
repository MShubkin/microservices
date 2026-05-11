INSERT INTO public.plan(
    uuid, status_id, commission_kind_id, purchasing_type_id, id, customer_id, contract_subject, sum_excluded_vat_rub,
    supplier_id,sum_excluded_vat,currency_id, delivery_start_date, delivery_end_date,created_by,changed_by,
    is_check_documentation, year, executor_method_id, pricing_expert_id, currency_rate, created_at, changed_at, pricing_sum_excluded_vat
)
VALUES
    ('12300000-0000-0000-0000-000000000001', 221, 1, 2, 123, 10, 'Разработка электропривода трубопроводной арматуры', 1504200,0,0,1,now()::date,now()::date,99,99,false,2024, 1, 999001,100,now()::date,now()::date, 1),
    ('12400000-0000-0000-0000-000000000001', 253, 1, 2, 124, 42, 'Разработка электропривода труброводной арматуры нормально-закрытого типа',10256087,0,0,1,now()::date,now()::date,99,99,false,2024, 1,999001,10,now()::date,now()::date, 1),
    ('12500000-0000-0000-0000-000000000001', 253, 1, 2, 125, 42, 'Разработка электропривода труброводной арматуры нормально-закрытого типа',10256087,0,0,1,now()::date,now()::date,99,99,false,2024, 1,999001,10,now()::date,now()::date, 1),
    ('12600000-0000-0000-0000-000000000001', 253, 1, 2, 126, 42, 'Разработка электропривода труброводной арматуры нормально-закрытого типа',10256087,0,0,1,now()::date,now()::date,99,99,false,2024, 1,999001,10,now()::date,now()::date, 1),
    ('12700000-0000-0000-0000-000000000001', 253, 1, 2, 127, 42, 'Разработка электропривода труброводной арматуры нормально-закрытого типа',10256087,0,0,1,now()::date,now()::date,99,99,false,2024, 1,999001,10,now()::date,now()::date, 1);

INSERT INTO public.plan_retrospective(
    plan_uuid, plan_id, plan_year, plan_status, id_ly, uuid_ly, is_removed
)
VALUES
    ('12300000-0000-0000-0000-000000000001', 123, 2021, 221, 124, '12400000-0000-0000-0000-000000000001', false),
    ('12300000-0000-0000-0000-000000000001', 123, 2021, 221, 125, '12500000-0000-0000-0000-000000000001', false),
    --- not exist in plan, but exist in sap_raw_headers
    ('14100000-0000-0000-0000-000000000001', 141, 2021, 221, 125, '14200000-0000-0000-0000-000000000001', false);


INSERT INTO public.status_history(
    uuid, object_uuid, status_id, comment, created_at, created_by)
VALUES
    ('110E8400E29B41D4A716446655440000', '12400000-0000-0000-0000-000000000001', 225, 'comment1', '2025-02-14 11:00:01', 1),
    ('220E8400E29B41D4A716446655440000', '12400000-0000-0000-0000-000000000001', 345, 'comment2',  '2025-02-14 11:00:02', 1),
    ('320E8400E29B41D4A716446655440000', '12400000-0000-0000-0000-000000000001', 111, 'comment3',  '2025-02-14 11:00:03', 1);