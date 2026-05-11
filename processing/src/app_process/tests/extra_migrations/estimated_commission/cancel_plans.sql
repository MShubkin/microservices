INSERT INTO plan(
    uuid,
    id,
    contract_subject,
    commission_kind_id,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    sum_excluded_vat_rub,
    currency_id,
    currency_rate,
    purchasing_type_id,
    status_id,
    delivery_start_date,
    delivery_end_date,
    pricing_organization_unit_id,
    section_id,
    created_by,
    changed_by,
    created_at,
    changed_at,
    reason_cancel_id,
    is_list_price) values
        ('00000000-0000-0000-0000-000000000001','1','Слишком много комаров',1,2,3,4,5,6,10,2,251,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,false),
        ('00000000-0000-0000-0000-000000000002','2','Слишком много комаров',1,2,3,4,5,6,10,2,0,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,false),
        ('00000000-0000-0000-0000-000000000003','3','Слишком много комаров',1,2,3,4,5,6,10,2,223,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,false),
        ('00000000-0000-0000-0000-000000000004','4','Трубы должны блестеть',1,2,3,4,5,6,10,2,225,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,false),
        ('00000000-0000-0000-0000-000000000005','5','Трубы должны блестеть',1,2,3,4,5,6,10,2,251,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,true),
        ('00000000-0000-0000-0000-000000000006','6','Трубы должны блестеть',1,2,3,4,5,6,10,7,251,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,false),
        ('00000000-0000-0000-0000-000000000007','7','Трубы должны блестеть',1,2,3,4,5,6,10,2,251,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,false),
        ('00000000-0000-0000-0000-000000000008','8','Трубы должны блестеть',1,2,3,4,5,6,10,2,251,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,false),
        ('00000000-0000-0000-0000-000000000009','9','Трубы должны блестеть',1,2,3,4,5,6,10,2,223,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,false),
        ('00000000-0000-0000-0000-000000000010','10','Трубы должны блестеть',1,2,3,4,5,6,10,7,225,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,false);

INSERT INTO contract_amendment(
    uuid,
    id,
    status_id,
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
    commission_kind_id,
    purchasing_type_id) values
        ('00000000-0000-0000-0001-000000000000',101,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
        ('00000000-0000-0000-0002-000000000000',102,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
        ('00000000-0000-0000-0003-000000000000',103,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
        ('00000000-0000-0000-0004-000000000000',104,353,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
        ('00000000-0000-0000-0005-000000000000',105,252,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
        ('00000000-0000-0000-0006-000000000000',106,253,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1);

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
        ('00000000-0000-0000-0000-000000000001','1',1,251,100,1,false,true,99,99,'1900-01-01','1900-01-01','1910-01-01'),
        ('00000000-0000-0000-0000-000000000002','2',1,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','1910-01-01'),
        ('00000000-0000-0000-0000-000000000003','3',2,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','1910-01-01');

INSERT INTO protocol_item(
    uuid,
    protocol_uuid,
    source_uuid,
    result_id,
    number,
    is_registered_by_d647,
    is_removed,
    created_at,
    changed_at,
    created_by,
    changed_by) values
        ('00000000-0000-0000-0000-000000000000','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000002',1,101,false,false,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000003',2,102,false,false,'1900-02-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000004',2,103,false,false,'1900-01-02','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000005',4,104,false,false,'1900-01-03','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000006',3,105,false,false,'1900-01-04','1900-01-01',99,99);

INSERT INTO agenda(
    uuid,
    id,
    meeting_date,
    status_id,
    pricing_organization_unit_id,
    is_removed,
    created_at,
    changed_at,
    created_by,
    changed_by) values
        ('00000000-0000-0000-0000-000000000001',1,'1900-01-01',100,2,false,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000002',2,'1900-01-02',200,2,false,'2000-01-01','1900-01-01',99,99),
        -- Повестка с неверным для аннулирования статусом
        ('00000000-0000-0000-0000-000000000003',3,'1900-01-02',400,2,false,'1900-01-01','1900-01-01',99,99);

INSERT INTO agenda_item(
    uuid,
    agenda_uuid,
    source_uuid,
    number,
    is_registered_by_d647,
    is_removed,
    is_excluded,
    created_at,
    changed_at,
    created_by,
    changed_by,
    reviewed_at) values
        ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','1',false,false,false,'1900-01-01','1900-01-01',99,99,'1900-01-01'),
        ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000007','1',false,false,false,'1900-01-01','1900-01-01',99,99,'1900-01-01'),
        ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000008','1',false,false,false,'1900-01-01','1900-01-01',99,99,'1900-01-01'),
        
        ('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0002-000000000000','1',false,false,false,'1900-01-01','1900-01-01',99,99,'1900-01-01'),
        ('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0002-000000000000','1',false,false,false,'1900-01-01','1900-01-01',99,99,'1900-01-01');

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
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001',1000,'труба, железная,',1000,0,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001',9000,'труба, железная,',9000,1,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000001',1000,'труба, железная,',1000,2,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000001',11000,'труба, железная,',11000,3,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000005', '00000000-0000-0000-0000-000000000001',1000,'труба, железная,',1000,4,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0000-000000000001',1000,'труба, железная,',1000,5,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000007', '00000000-0000-0000-0000-000000000002',1000,'пёс',1000,6,6,2,3,4,5,false,99,99,2,10,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000008', '00000000-0000-0000-0000-000000000002',1000,'пёс',1000,7,7,2,3,4,5,false,99,99,2,10,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000009', '00000000-0000-0000-0000-000000000002',1000,'пёс',1000,8,8,2,3,4,5,false,99,99,2,10,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000010', '00000000-0000-0000-0000-000000000002',1000,'пёс',1000,9,9,2,3,4,5,false,99,99,3,9,now()::date,now()::date),
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
