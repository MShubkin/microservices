INSERT INTO protocol(
    uuid,
    id,
    protocol_type_id,
    registration_number,
    status_id,
    pricing_organization_unit_id,
    is_secret,
    is_removed,
    created_by,
    changed_by,
    created_at,
    changed_at,
    protocol_date) values
        ('00000000-0000-0000-0000-000000000001','1',1,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','1900-01-10');

INSERT INTO protocol_item(
    uuid,
    protocol_uuid,
    source_uuid,
    result_id,
    number,
    is_registered_by_d647,
    is_excluded,
    is_removed,
    created_at,
    changed_at,
    created_by,
    changed_by) values
        ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001',1,1001,false,false,false,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000002',2,1002,false,false,false,'1900-02-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000003',3,1003,true,false,false,'1900-01-02','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000004',4,1004,true,false,false,'1900-01-03','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0001-000000000000',1,1004,false,false,false,'1900-01-03','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0002-000000000000',2,1002,false,false,true,'1900-02-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000007','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0003-000000000000',3,1002,true,true,false,'1900-02-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000008','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0004-000000000000',4,1004,true,false,false,'1900-01-03','1900-01-01',99,99);

INSERT INTO plan(
    uuid,
    id,
    contract_subject,
    commission_kind_id,
    commission_date,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    sum_excluded_vat_rub,
    currency_id,
    purchasing_type_id,
    status_id,
    delivery_start_date,
    delivery_end_date,
    created_by,
    changed_by,
    pricing_organization_unit_id,
    currency_rate,
    created_at,changed_at,is_not_purchase, reason_cancel_id
) values
        ('00000000-0000-0000-0000-000000000001','1','Должен стать 140',1,now()::date,2,3,4,5,6,2,251,now()::date,now()::date,99,99,1,1,now()::date,now()::date, false,0),
        ('00000000-0000-0000-0000-000000000002','2','Должен стать 140',1,now()::date,2,3,4,5,6,2,251,now()::date,now()::date,99,99,1,1,now()::date,now()::date, false,0),
        ('00000000-0000-0000-0000-000000000003','3','Должен стать 160 так как is_not_purchase=true',1,now()::date,2,3,4,5,6,2,251,now()::date,now()::date,99,99,1,1,now()::date,now()::date, true,0),
        ('00000000-0000-0000-0000-000000000004','4','Должен стать 140',1,now()::date,2,3,4,5,6,2,251,now()::date,now()::date,99,99,1,1,now()::date,now()::date, false,0);

INSERT INTO contract_amendment(
    uuid,
    id,
    status_id,
    commission_date,
    contract_subject,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    currency_id, 
    pricing_expert_id, 
    pricing_resume, 
    section_id,
    created_at,
    changed_at,
    created_by,
    changed_by,
    commission_kind_id,purchasing_type_id,
    delta_sum_excluded_vat, 
    pricing_delta_sum_excluded_vat, 
    pricing_organization_unit_id) values
        ('00000000-0000-0000-0001-000000000000',101,251,now()::date,'Должен стать ', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1, 3, 4, 2),
        ('00000000-0000-0000-0002-000000000000',102,252,now()::date,'Должен стать ', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1, 3, 4, 2),
        ('00000000-0000-0000-0003-000000000000',103,252,now()::date,'Должен стать 140', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1, 3, 4, 2),
        ('00000000-0000-0000-0004-000000000000',104,252,now()::date,'Должен стать 140', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1, 3, 4, 2);

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


INSERT INTO public.plan_item(
    uuid,plan_uuid,category_id,description_internal,quantity,
    id,product_type_id,budget_item_id,okved2_id,unit_id,payment_balance_item_id,is_not_russian_delivery,created_by,changed_by,currency_id,currency_rate,
    created_at, changed_at
    )
    VALUES 
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000003',1000,'труба, железная,',1000,0,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000003',9000,'труба, железная,',9000,1,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000003',1000,'труба, железная,',1000,2,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000003',11000,'труба, железная,',11000,3,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000005', '00000000-0000-0000-0000-000000000003',1000,'труба, железная,',1000,4,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0000-000000000003',1000,'труба, железная,',1000,5,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000007', '00000000-0000-0000-0000-000000000004',1000,'пёс',1000,6,6,2,3,4,5,false,99,99,2,10,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000008', '00000000-0000-0000-0000-000000000004',1000,'пёс',1000,7,7,2,3,4,5,false,99,99,2,10,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000009', '00000000-0000-0000-0000-000000000004',1000,'пёс',1000,8,8,2,3,4,5,false,99,99,2,10,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000010', '00000000-0000-0000-0000-000000000004',1000,'пёс',1000,9,9,2,3,4,5,false,99,99,3,9,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000011', '00000000-0000-0000-0000-000000000002',1000,'пёс',1000,10,10,2,3,4,5,false,99,99,1,1,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000012', '00000000-0000-0000-0000-000000000002',1000,'пёс',1000,11,11,2,3,4,5,false,99,99,1,1,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000013', '00000000-0000-0000-0000-000000000002',1000,'такса, железная,',1000,12,1,2,3,4,5,false,99,99,1,1,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000014', '00000000-0000-0000-0000-000000000002',1000,'такса, железная,',1000,13,1,2,3,4,5,false,99,99,1,1,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000015', '00000000-0000-0000-0000-000000000002',1000,'такса, железная,',1000,14,1,2,3,4,5,false,99,99,1,1,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000016', '00000000-0000-0000-0000-000000000002',1000,'такса, железная,',1000,15,1,2,3,4,5,false,99,99,1,1,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000017', '00000000-0000-0000-0000-000000000002',1000,'такса, железная,',1000,16,1,2,3,4,5,false,99,99,1,1,now()::date,now()::date);

ALTER TABLE plan_item_version DROP COLUMN pricing_version;
ALTER TABLE plan_item_version ADD COLUMN pricing_version SMALLINT NOT NULL;
INSERT INTO plan_item_version (SELECT *,1 FROM plan_item);
