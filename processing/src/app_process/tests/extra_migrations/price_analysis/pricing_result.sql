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
    savings_sum_included_vat_rub,
    pricing_sum_included_vat_rub,
    savings_accounting_id
    ) values
        ('00000000-0000-0000-0000-000000000001',1,'Слишком много комаров',1,2,3,4,5,6,10,2,222,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,1000,500,2),
        ('00000000-0000-0000-0000-000000000002',2,'Слишком много комаров',1,2,3,4,5,6,10,2,342,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,0,0,2),
        ('00000000-0000-0000-0000-000000000003',3,'Слишком много комаров',1,2,3,4,5,6,10,2,352,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,1500,1500,2),
        ('00000000-0000-0000-0000-000000000004',4,'Трубы должны блестеть',1,2,3,4,5,6,10,2,221,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,20000,30000,2);

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
    purchasing_type_id,
    savings_sum_included_vat_rub,
    pricing_sum_included_vat_rub,
    savings_accounting_id
    ) values
        ('00000000-0000-0000-0001-000000000000',11,222,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1, 1000, 500, 2),
        ('00000000-0000-0000-0002-000000000000',12,342,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1, 0, 0, 2),
        ('00000000-0000-0000-0003-000000000000',13,352,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1, 1500, 1500, 2),
        ('00000000-0000-0000-0004-000000000000',14,221,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1, 20000, 25000, 2);

INSERT INTO status_history (uuid, object_uuid, status_id, comment, created_at, created_by)
VALUES
('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 222, 'ppz status 222', '2021-09-30 11:21:12.877345', 123),
('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', 342, 'pzz status 342', '2022-09-30 12:22:26.912647', 123),
('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000001', 352, 'pzz status 352', '2023-09-30 13:23:26.962647', 123),
('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000001', 223, 'pzz next status 223', '2023-11-30 13:23:26.962647', 123),
('00000000-0000-0000-0000-000000000005', '00000000-0000-0000-0000-000000000001', 225, 'pzz next status 225', '2024-11-30 14:23:26.962647', 123),
('00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0001-000000000000', 222, 'dc status 222', '2021-10-01 06:43:09.963269', 123),
('00000000-0000-0000-0000-000000000007', '00000000-0000-0000-0001-000000000000', 342, 'dc status 342', '2022-10-02 07:43:09.963269', 123),
('00000000-0000-0000-0000-000000000008', '00000000-0000-0000-0001-000000000000', 352, 'dc status 352', '2024-12-22 08:43:09.961249', 123);

INSERT INTO public.regulatory_deadline_price (
    uuid,
    section,
    type_criticality,
    color_scheme_id,
    start_day,
    end_day,
    created_by,
    created_at,
    changed_by,
    changed_at,
    status
) VALUES 
    ('00000000-0000-0000-0000-000000000001', 10, 33, 1, 0, 1, 101, NOW(), 101, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000002', 10, 33, 3, 1, 10, 101, NOW(), 102, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000003',  7, 33, 1, 0, 5, 101, NOW(), 101, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000004',  7, 33, 2, 6, 7, 101, NOW(), 101, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000005',  7, 33, 3, 8, 10, 101, NOW(), 101, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000006',  8, 33, 1, 0, 5, 101, NOW(), 101, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000007',  8, 33, 2, 6, 7, 101, NOW(), 101, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000008',  8, 33, 3, 8, 10, 101, NOW(), 101, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000009',  8, 33, 1, 1, 3, 101, NOW(), 101, NOW(), TRUE);

