INSERT INTO plan(
    uuid,
    id,
    status_id,
    contract_subject,
    commission_kind_id,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    sum_excluded_vat_rub,
    currency_id,
    currency_rate,
    purchasing_type_id,
    delivery_start_date,
    delivery_end_date,
    created_by,
    changed_by,
    created_at,
    changed_at
) values
    ('00000000-0000-0000-0000-000000000001',1000000001,251,    'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',99,99,now()::timestamp,now()::timestamp),
    ('00000000-0000-0000-0000-000000000002',1000000002,252,    'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',99,99,now()::timestamp,now()::timestamp),
    ('00000000-0000-0000-0000-000000000003',1000000003,253,    'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',99,99,now()::timestamp,now()::timestamp);

INSERT INTO agenda
(uuid, id, meeting_date, status_id, pricing_organization_unit_id, is_removed, created_at, changed_at, created_by, changed_by) VALUES
    ('00000000-0000-0000-0000-000000000001',1,'2024-07-01',100,1,false,'2024-07-01','2024-07-01',99,99),
    ('00000000-0000-0000-0000-000000000002',2,'2024-07-02',200,2,false,'2024-07-01','2024-07-01',99,100);

INSERT INTO protocol
    (uuid, id, protocol_type_id, registration_number, status_id, pricing_organization_unit_id, is_secret, is_removed, protocol_date, created_by, changed_by, created_at, changed_at) VALUES
    ('00000000-0000-0000-0000-000000000001','1',1,100,100,1,false,false,'2024-07-01',99,99,'2024-07-01','2024-07-01'),
    ('00000000-0000-0000-0000-000000000002','2',1,200,200,1,false,false,'2024-07-01',99,99,'2024-07-01','2024-07-01'),
    ('00000000-0000-0000-0000-000000000003','3',1,300,300,1,false,false,'2024-07-01',99,99,'2024-07-01','2024-07-01'),

    ('00000000-0000-0000-0000-000000000004','4',2,100,100,2,false,false,'2024-07-01',99,99,'2024-07-01','2024-07-01'),
    ('00000000-0000-0000-0000-000000000005','5',2,200,200,2,false,false,'2024-07-01',99,99,'2024-07-01','2024-07-01'),
    ('00000000-0000-0000-0000-000000000006','6',2,300,300,2,false,false,'2024-07-01',99,99,'2024-07-01','2024-07-01');