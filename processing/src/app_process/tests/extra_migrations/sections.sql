INSERT INTO plan(
    uuid,
    id,
    commission_kind_id,
    purchasing_type_id,
    status_id,
    year,
    is_check_documentation,
    pricing_expert_id,
    commission_date,
    pricing_started_at,

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
        ('00000000-0000-0000-0000-000000000001', 1,  1, 2, 221, date_part('year', now()), false, 1,    now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-9999-0000-000000000001', 1,  1, 2, 221, date_part('year', now()), false, 1,    now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,false),
        ('00000000-0000-0000-0000-000000000002', 2,  1, 2, 221, 0,                        false, 1,    now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000003', 3,  1, 2, 251, 0,                        false, 1,    now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000004', 4,  3, 2, 253, 0,                        false, 1,    now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000005', 5,  3, 1, 253, 0,                        false, 1,    now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000006', 6,  2, 2, 252, 0,                        false, 1,    now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000007', 7,  2, 2, 252, 0,                        false, 0,    now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000008', 8,  2, 2, 251, 0,                        false, 0,    now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000009', 9,  0, 0, 222, date_part('year', now()), true,  0,    now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000010', 10, 0, 0, 222, 0,                        false, null, now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000011', 11, 0, 0, 223, date_part('year', now()), false, null, now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,'2024-11-27 15:00:00'::timestamp,now()::timestamp,true),
        ('00000000-0000-0000-0000-000000000012', 12, 0, 0, 223, 0,                        false, null, now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,true),
        ('00000000-0000-9999-0000-000000000012', 12, 0, 0, 223, 0,                        false, null, now(), NOW() - INTERVAL '7 days',                   2,3,100,5,6,10,'Слишком мало комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,false);

INSERT INTO contract_amendment(
    uuid,
    id,
    commission_kind_id,
    purchasing_type_id,
    status_id,
    year,
    is_check_documentation,
    pricing_expert_id,
    commission_date,
    pricing_started_at,

    contract_subject,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    delta_sum_excluded_vat,
    pricing_sum_excluded_vat,
    pricing_delta_sum_excluded_vat,
    currency_id,  
    pricing_resume, section_id,
    created_at,
    changed_at,
    created_by,
    changed_by,
    is_actual
    ) values
        ('00000000-0000-0000-0001-000000000000', 1,  1, 2, 221, date_part('year', now()), true,  1,    now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-9999-0001-000000000000', 1,  1, 2, 221, date_part('year', now()), true,  1,    now(), NOW() - INTERVAL '7 days',      'Not sub', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,false),
        ('00000000-0000-0000-0002-000000000000', 2,  1, 2, 221, 0,                        false, 1,    now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-0000-0003-000000000000', 3,  1, 2, 251, 0,                        false, 1,    now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-0000-0004-000000000000', 4,  3, 2, 253, 0,                        false, 1,    now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-0000-0005-000000000000', 5,  3, 2, 253, 0,                        false, 1,    now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-0000-0006-000000000000', 6,  2, 2, 252, 0,                        false, 1,    now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 111, 666, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-0000-0007-000000000000', 7,  2, 2, 252, 0,                        false, 0,    now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 222, 777, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-0000-0008-000000000000', 8,  1, 2, 252, 0,                        false, 0,    now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-0000-0009-000000000000', 9,  0, 0, 222, date_part('year', now()), true,  0,    now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-0000-0010-000000000000', 10, 0, 0, 222, date_part('year', now()), false, 0,    now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-0000-0011-000000000000', 11, 0, 0, 223, date_part('year', now()), false, null, now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-0000-0012-000000000000', 12, 0, 0, 223, 0,                        false, null, now(), NOW() - INTERVAL '7 days',      'Subject', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,true),
        ('00000000-0000-9999-0012-000000000000', 12, 0, 0, 223, 0,                        false, null, now(), NOW() - INTERVAL '7 days',      'Not sub', 1, 1, 100, 200, 300, 400, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99,false);

INSERT INTO public.agenda_item(
    uuid,
    agenda_uuid,
    source_uuid,
    "number",
    is_registered_by_d647,
    is_removed,
    is_excluded,
    created_at,
    changed_at,
    created_by,
    changed_by
) VALUES
        ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000003',1,false,false,false,now()::date,'1970-04-23',99,99),
        ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0003-000000000000',2,false,false,false,now()::date,'1990-04-23',99,99),
        ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000003','00000000-0000-0000-0002-000000000000',2,false,false,false,now()::date,'1990-04-23',99,99);

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
        ('00000000-0000-0000-0000-000000000001',1,'2000-01-01',100,2, false,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000002',2,'2000-01-01',100,2, false,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000003',3,'2000-01-01',100,2, false,'1900-01-01','1900-01-01',99,99);

INSERT INTO public.protocol_item(
    uuid,
    protocol_uuid,
    source_uuid,
    "number",
    sum_excluded_vat,
    commission_sum_excluded_vat,
    is_registered_by_d647,
    is_removed,
    result_id,
    created_at,
    changed_at,
    created_by,
    changed_by
) VALUES
    ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000003',1,200,100,false,false,4,now()::date,'1970-03-23',99,99),
    ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0003-000000000000',2,300,200,false,false,4,now()::date,'1970-04-23',99,99),
    ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000003','00000000-0000-0000-0002-000000000000',2,400,300,false,false,2,now()::date,'1970-04-23',99,99),
    ('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000008',2,200,100,false,false,4,now()::date,'1970-04-23',99,99),
    ('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000005','00000000-0000-0000-0008-000000000000',2,200,100,false,false,4,now()::date,'1970-04-23',99,99),
    ('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000001',2,200,100,false,false,4,now()::date,'1970-04-23',99,99);

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
        ('00000000-0000-0000-0000-000000000001','1',1,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','2000-11-11'),
        ('00000000-0000-0000-0000-000000000002','2',1,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','2001-11-11'),
        ('00000000-0000-0000-0000-000000000003','3',1,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','2002-11-11'),
        ('00000000-0000-0000-0000-000000000004','4',2,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','2003-11-11'),
        ('00000000-0000-0000-0000-000000000005','5',2,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','2004-11-11'),
        ('00000000-0000-0000-0000-000000000006','6',2,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','2004-11-11');


INSERT INTO status_history (uuid, object_uuid, status_id, comment, created_at, created_by)
VALUES
('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 221, 'Comment1', '2021-09-30 11:20:12.877345', 123),
('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', 221, 'Comment2', '2022-09-30 12:22:26.922647', 123),
('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0001-000000000000', 221, 'Comment3', '2021-10-01 06:43:09.963269', 123),
('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0001-000000000000', 221, 'Comment4', '2022-10-02 07:43:09.963269', 123),
('00000000-0000-0000-0000-000000000005', '00000000-0000-0000-0001-000000000000', 223, 'start_approved_date', '2022-10-02 10:40:09.964269', 123),
('00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0001-000000000000', 222, 'start_determine_price_date', '2021-10-05 10:40:09.964269', 123),
('00000000-0000-0000-0000-000000000007', '00000000-0000-0000-0001-000000000000', 342, 'start_determine_price_date', '2020-11-19 10:40:11.964269', 123),
('00000000-0000-0000-0000-000000000008', '00000000-0000-0000-0000-000000000001', 222, 'start_determine_price_date', '2024-11-24 10:40:10.964269', 123),
('00000000-0000-0000-0000-000000000009', '00000000-0000-0000-0000-000000000001', 352, 'start_determine_price_date', '2022-10-06 10:40:12.964269', 123),
('00000000-0000-0000-0000-000000000010', '00000000-0000-0000-0000-000000000011', 352, 'number_of_days_with_expert_threshold', '2024-10-05 10:40:11.964269', 123),
('00000000-0000-0000-0000-000000000011', '00000000-0000-0000-0000-000000000011', 352, 'number_of_days_with_expert_threshold', '2024-11-26 10:40:12.964269', 123),
('00000000-0000-0000-0000-000000000012', '00000000-0000-0000-0000-000000000012', 352, 'number_of_days_with_expert_threshold', '2024-11-20 10:40:11.964269', 123),
('00000000-0000-0000-0000-000000000014', '00000000-0000-0000-0011-000000000000', 352, 'number_of_days_with_expert_threshold', '2024-11-25 10:40:12.964269', 123),
('00000000-0000-0000-0000-000000000015', '00000000-0000-0000-0011-000000000000', 352, 'number_of_days_with_expert_threshold', '2024-10-06 10:40:12.964269', 123),
('00000000-0000-0000-0000-000000000016', '00000000-0000-0000-0012-000000000000', 352, 'number_of_days_with_expert_threshold', '2024-10-05 10:40:12.964269', 123),
('00000000-0000-0000-0000-000000000017', '00000000-0000-0000-0012-000000000000', 352, 'number_of_days_with_expert_threshold', '2024-10-06 10:40:12.964269', 123);

TRUNCATE public.regulatory_deadline_price;
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
    ('00000000-0000-0000-0000-000000000006',  8, 33, 1, 0, 1, 101, NOW(), 101, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000007',  8, 33, 2, 6, 7, 101, NOW(), 101, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000008',  8, 33, 3, 8, 10, 101, NOW(), 101, NOW(), FALSE),
    ('00000000-0000-0000-0000-000000000009',  8, 33, 1, 1, 3, 101, NOW(), 101, NOW(), TRUE);
