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
    previous_sum_vat,
    is_removed,
    kind_id
) VALUES
('00000001-0000-0000-0000-000000000001', '00000001-0000-0000-0000-000000000001', 100, now(), now(), 99, 99, 0, 0,
643, 100000, 100000, 2000, 6, 100000, 120000, 20000, false, 1
),
('00000001-0000-0000-0000-000000000002', '00000001-0000-0000-0000-000000000001', 200, now(), now(), 99, 99, 0, 0,
643, 100000, 300000, 1000, 3, 600000, 660000, 60000, false, 1
),
('00000001-0000-0000-0000-000000000003', '00000001-0000-0000-0000-000000000001', 300, now(), now(), 99, 99, 0, 0,
646, 3700, 10000, 2000, 6, 10000, 12000, 2000, true, 1
),
('00000001-0000-0000-0000-000000000004', '00000001-0000-0000-0000-000000000001', 400, now(), now(), 99, 99, 0, 0,
414, 32944000, 30000, 1000, 3, 60000, 66000, 6000, false, 8
);

INSERT INTO attachment(
  uuid,
  object_uuid,
  "number",
  parent_number,
  "size",
  created_at,
  changed_at,
  created_by,
  changed_by,
  category_id
) VALUES
    ('00000000-0000-0000-0000-000000000001','00000001-0000-0000-0000-000000000001',5,1,54,'1999-1-1','1999-1-1',1,1,1),
    ('00000000-0000-0000-0000-000000000003','00000001-0000-0000-0000-000000000001',5,1,3453,'1999-1-1','1999-1-1',1,1,1),
    ('00000000-0000-0000-0000-000000000006','00000001-0000-0000-0000-000000000001',3,1,345345,'1999-1-1','1999-1-1',1,1,2),
    ('00000000-0000-0000-0000-000000000009','00000001-0000-0000-0000-000000000001',2,1,100,'1999-1-1','1999-1-1',1,1,2);
