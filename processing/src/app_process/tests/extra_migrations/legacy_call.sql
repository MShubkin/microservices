INSERT INTO plan(
    uuid,
    id,
    commission_kind_id,
    purchasing_type_id,
    status_id,
    year,
    check_documentation_date,
    is_check_documentation,
    pricing_expert_id,

    customer_id,
    supplier_id,
    sum_excluded_vat,
    sum_excluded_vat_rub,
    currency_id,
    currency_rate,
    contract_subject,
    delivery_start_date,
    delivery_end_date,
    pricing_organization_unit_id,
    section_id,
    created_by,
    changed_by,
    created_at,
    changed_at,
    is_actual
    ) values
        ('00000000-0000-0000-0000-000000000001', 1,  1, 2, 223, date_part('year', now()),'2000-01-01 05:05:05', false, 1,                       2,3,4,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000002', 2,  1, 2, 223, 0, '2000-01-01 05:05:05',                      false, 1,                       2,3,4,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true);

INSERT INTO contract_amendment(
    uuid,
    id,
    commission_kind_id,
    purchasing_type_id,
    status_id,
    year,
    is_check_documentation,
    pricing_expert_id,

    contract_subject,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    currency_id,  
    pricing_resume,
    section_id,
    created_at,
    changed_at,
    created_by,
    changed_by
    ) values
        ('00000000-0000-0000-0001-000000000000', 1,  1, 2, 223, date_part('year', now()), true,  1,                        'Subject', 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99),
        ('00000000-0000-0000-0002-000000000000', 2,  1, 2, 223, 0,                        false, 1,                        'Subject', 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99);

-- Version updates fail if there are no versions, so we create them manually.
ALTER TABLE plan_version DROP COLUMN pricing_version;
ALTER TABLE plan_version ADD COLUMN pricing_version SMALLINT NOT NULL;
INSERT INTO plan_version (SELECT *,1 FROM plan);
INSERT INTO contract_amendment_version(
    uuid,
    id,
    commission_kind_id,
    purchasing_type_id,
    status_id,
    year,
    is_check_documentation,
    pricing_expert_id,
    contract_subject,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    currency_id,  
    pricing_resume,
    section_id,
    created_at,
    changed_at,
    created_by,
    changed_by,
    pricing_version
) (SELECT 
    uuid,
    id,
    commission_kind_id,
    purchasing_type_id,
    status_id,
    year,
    is_check_documentation,
    pricing_expert_id,
    contract_subject,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    currency_id,  
    pricing_resume,
    section_id,
    created_at,
    changed_at,
    created_by,
    changed_by,1 FROM contract_amendment);

INSERT INTO plan_item(
    uuid,
    plan_uuid,
    id,
    category_id,
    product_type_id,
    budget_item_id,
    okved2_id,
    unit_id,
    payment_balance_item_id,
    quantity,
    is_not_russian_delivery,
    created_by,
    changed_by,
    currency_id,
    currency_rate,
    pricing_unit_id,
    created_at,
    changed_at) values
('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000007',1001,1,2,3,4,5,6,7,true,99,99,1,1, 1,now(), now()),
('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000007',1002,1,2,3,4,5,6,7,true,99,99,1,1, 1,now(), now()),
('40000000-0000-0000-0000-000000000000','00000000-0000-0000-0000-000000000001',1003,1,2,3,4,5,6,7,true,99,99,1,1, 1,now(), now()),
('30000000-0000-0000-0000-000000000000','99000000-0000-0000-0000-000000000099',1004,1,2,3,4,5,6,7,true,99,99,1,1, 1,now(), now());

ALTER TABLE plan_item_version DROP COLUMN pricing_version;
ALTER TABLE plan_item_version ADD COLUMN pricing_version SMALLINT NOT NULL;
INSERT INTO plan_item_version (SELECT *,1 FROM plan_item);

INSERT INTO document_approver (uuid,
    document_uuid, plan_id, department_id, number, planned_date,
    division_id, division_assigned_at, expert_id, responded_at, status_appr, responsible_person_id,
    is_auto, route_id, send_date_1, is_preapproved, is_removed, is_actual,
    created_at, created_by, changed_at, changed_by
) VALUES
('00000000-0000-0000-0001-000000000001','00000000-0000-0000-0000-000000000001',1, 10, 1, '2025-09-13',
 101, '2024-09-12', 777, '2024-09-13', 2, 888,
 false, ARRAY[]::BIGINT[], '2024-12-31', false, false, false,
 now(), 666, now(), 555),
('00000000-0000-0000-0001-000000000002','00000000-0000-0000-0000-000000000001',1, 10, 1, '2025-09-13',
 null, null, 777, '2024-09-13', 2, 888,
 false, ARRAY[]::BIGINT[], '2024-12-31', false, false, true,
 now(), 666, now(), 555),
('00000000-0000-0000-0001-000000000003','00000000-0000-0000-0000-000000000002',2, 10, 2, '2025-09-13',
 null, null, 777, '2024-09-13', 2, 888,
 false, ARRAY[]::BIGINT[], '2024-12-31', false, false, true,
 now(), 666, now(), 555);
