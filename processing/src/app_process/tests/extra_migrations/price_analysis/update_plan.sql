INSERT INTO plan(
    uuid,
    id,
    commission_kind_id,
    purchasing_type_id,
    status_id,
    year,    
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
    changed_at
    ) values
        ('00000000-0000-0000-0000-000000000001', 1,  1, 2, 221, date_part('year', now()), 1,                       2,3,4,5,643,100000,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp),
        ('00000000-0000-0000-0000-000000000002', 2,  1, 2, 221, 0,                        1,                       2,3,4,5,643,100000,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp),
        ('00000000-0000-0000-0000-000000000003', 3,  1, 2, 221, 0,                        1,                       2,3,4,5,643,100000,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp);
-- Version updates fail if there are no versions, so we create them manually.
ALTER TABLE plan_version DROP COLUMN pricing_version;
ALTER TABLE plan_version ADD COLUMN pricing_version SMALLINT NOT NULL;
INSERT INTO plan_version (SELECT *,1 FROM plan);

INSERT INTO public.plan_item(
    uuid,plan_uuid,category_id,description_internal,quantity,
    id,product_type_id,budget_item_id,okved2_id,unit_id,payment_balance_item_id,is_not_russian_delivery,created_by,changed_by,currency_id,currency_rate,
    created_at, changed_at
    )
    VALUES 
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001',1000,'труба, железная,',1000,0,1,2,3,4,5,false,99,99,643,100000,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001',9000,'труба, железная,',9000,1,1,2,3,4,5,false,99,99,643,100000,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000003',10001,'труба, железная,',9000,1,1,2,3,4,5,false,99,99,646,3700,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000003',10002,'труба, железная,',9000,1,1,2,3,4,5,false,99,99,414,32944000,now()::date,now()::date);

INSERT INTO attachment (
  uuid,
  object_uuid,
  "number",
  parent_number,
  "size",
  created_at,
  changed_at,
  created_by,
  changed_by
) VALUES
    ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001',5,1,54,'1999-1-1','1999-1-1',1,1),
    ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000001',4,1,3453,'1999-1-1','1999-1-1',1,1),
    ('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000001',3,1,345345,'1999-1-1','1999-1-1',1,1),
    ('00000000-0000-0000-0000-000000000009','00000000-0000-0000-0000-000000000099',2,1,100,'1999-1-1','1999-1-1',1,1);

INSERT INTO contract_amendment(
    uuid,
    id,
    contract_subject,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    currency_id,
    currency_rate,
    status_id,
    created_by,
    changed_by,
    created_at,
    changed_at,
    pricing_expert_id,
    pricing_organization_unit_id,
    pricing_resume,
    section_id,
    commission_kind_id,
    purchasing_type_id,
    pricing_sum_excluded_vat) VALUES
        ('00000001-0000-0000-0000-000000000001',101,'InPerson registered',7,3,4,643,100000,150,99,99,now()::date,now()::date, 1, 1, 'Resume', 1, 1, 1, 1),
        ('00000001-0000-0000-0000-000000000002',102,'InPerson registered',7,3,4,643,100000,150,99,99,now()::date,now()::date, 1, 1, 'Resume', 1, 1, 1, 1);

INSERT INTO contract_amendment_item(
    uuid, header_uuid, id, created_at, changed_at, created_by, changed_by, pricing_unit_id, pricing_currency_id,
    currency_id,
    currency_rate,
    previous_price,
    previous_quantity,
    previous_vat_id,
    previous_sum_excluded_vat,
    previous_sum_included_vat,
    previous_sum_vat
) VALUES
('00000001-0000-0000-0000-000000000001', '00000001-0000-0000-0000-000000000001', 100, now(), now(), 99, 99, 0, 0,
643, 100000, 100000, 2000, 6, 100000, 120000, 20000
),
('00000001-0000-0000-0000-000000000002', '00000001-0000-0000-0000-000000000001', 200, now(), now(), 99, 99, 0, 0,
643, 100000, 300000, 1000, 3, 600000, 660000, 60000
),
('00000001-0000-0000-0000-000000000003', '00000001-0000-0000-0000-000000000002', 300, now(), now(), 99, 99, 0, 0,
646, 3700, 10000, 2000, 6, 10000, 12000, 2000
),
('00000001-0000-0000-0000-000000000004', '00000001-0000-0000-0000-000000000002', 400, now(), now(), 99, 99, 0, 0,
414, 32944000, 30000, 1000, 3, 60000, 66000, 6000
);

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
        ('00000000-0000-0000-0000-000000000001','1',1,253,100,1,false,false,99,99,'1900-01-01','1900-01-01','1910-01-01'),
        ('00000000-0000-0000-0000-000000000002','2',1,253,100,1,false,true,99,99,'1900-01-01','1900-01-01','1910-01-01');

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
        ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001',2,102,false,true,'1900-01-02','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000001','00000001-0000-0000-0000-000000000001',1,103,false,false,'1900-01-03','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000002','00000001-0000-0000-0000-000000000002',2,103,false,false,'1900-01-03','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000002',2,103,false,false,'1900-01-03','1900-01-01',99,99);
