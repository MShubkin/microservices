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
    pricing_created_at,
    is_actual
    ) values
        ('00000000-0000-0000-0000-000000000001', 1,  1, 2, 223, date_part('year', now()),'2000-01-01 05:05:05', false, 1,                       2,3,400,500,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,'1999-09-09','1999-09-09','1999-09-09', true),
        ('00000000-0000-0000-0000-000000000002', 2,  1, 2, 223, 0, '2000-01-01 05:05:05',                      false, 1,                       2,3,400,500,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,now()::timestamp,true);

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
    sum_excluded_vat_rub,
    currency_id,  
    pricing_resume,
    section_id,
    created_at,
    changed_at,
    pricing_created_at,
    created_by,
    changed_by
    ) values
        ('00000000-0000-0000-0001-000000000000', 1,  1, 2, 223, date_part('year', now()), true,  1,                        'Subject', 1, 1, 100, 200, 1, 'Resume', 1, now()::timestamp,now()::timestamp,'1999-09-09', 99,99),
        ('00000000-0000-0000-0002-000000000000', 2,  1, 2, 223, 0,                        false, 1,                        'Subject', 1, 1, 100, 200, 1, 'Resume', 1, now()::timestamp,now()::timestamp,'1999-09-09', 99,99);

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
    sum_excluded_vat_rub,
    currency_id,  
    pricing_resume,
    section_id,
    created_at,
    changed_at,
    pricing_created_at,
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
    sum_excluded_vat_rub,
    currency_id,  
    pricing_resume,
    section_id,
    created_at,
    changed_at,
    pricing_created_at,
    created_by,
    changed_by,1 FROM contract_amendment);

INSERT INTO plan_item(
    uuid,
    plan_uuid,
    id,
    description_internal,
    number,
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
    created_at,
    changed_at) values
('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000007',1001,'plan item 1',1,1,2,3,4,5,6,7,true,99,99,1,1, now(), now()),
('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000007',1002,'plan item 2',2,1,2,3,4,5,6,7,true,99,99,1,1, now(), now()),
('40000000-0000-0000-0000-000000000000','00000000-0000-0000-0000-000000000001',1003,'plan item 3',3,1,2,3,4,5,6,7,true,99,99,1,1, now(), now()),
('30000000-0000-0000-0000-000000000000','99000000-0000-0000-0000-000000000099',1004,'plan item 4',4,1,2,3,4,5,6,7,true,99,99,1,1, now(), now());

ALTER TABLE plan_item_version DROP COLUMN pricing_version;
ALTER TABLE plan_item_version ADD COLUMN pricing_version SMALLINT NOT NULL;
INSERT INTO plan_item_version (SELECT *,1 FROM plan_item);

INSERT INTO contract_amendment_item(
    uuid,
    header_uuid,
    id,
    description_internal,
    number,
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
    pricing_currency_id,
    created_at,
    changed_at) values
('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0001-000000000000',1001,'ca item 1',1,1,2,3,4,5,6,7,true,99,99,1,1, 2,3, now(), now()),
('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0002-000000000000',1002,'ca item 2',2,1,2,3,4,5,6,7,true,99,99,1,1, 2,3, now(), now()),
('40000000-0000-0000-0000-000000000000','00000000-0000-0000-0001-000000000000',1003,'ca item 3',3,1,2,3,4,5,6,7,true,99,99,1,1, 2,3, now(), now()),
('30000000-0000-0000-0000-000000000000','00000000-0000-0000-0002-000000000000',1004,'ca item 4',4,1,2,3,4,5,6,7,true,99,99,1,1, 2,3, now(), now());

ALTER TABLE contract_amendment_item_version DROP COLUMN pricing_version;
ALTER TABLE contract_amendment_item_version ADD COLUMN pricing_version SMALLINT NOT NULL;
INSERT INTO contract_amendment_item_version (SELECT *,1 FROM contract_amendment_item);
