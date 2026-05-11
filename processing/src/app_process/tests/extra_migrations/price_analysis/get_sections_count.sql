INSERT INTO price_analysis_user (
    id,
    type_user_id,
    start_date,
    end_date,
    created_at,
    changed_at,
    created_by,
    changed_by
) values
    (1, 1, now()::timestamp, now()::timestamp, now()::timestamp, now()::timestamp, 1, 1),
    (2, 2, now()::timestamp, now()::timestamp, now()::timestamp, now()::timestamp, 1, 1),
    (3, 2, now()::timestamp, now()::timestamp, now()::timestamp, now()::timestamp, 1, 1);

INSERT INTO plan(
    uuid,
    id,
    status_id,
    pricing_expert_id,
    pricing_organization_unit_id,
    is_check_documentation,
    is_actual,
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
    section_id,
    created_by,
    changed_by,
    created_at,
    changed_at
) values
    ('00000000-0000-0000-0000-000000000001',1000000001,221,2,1,false,true,    'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,99,99,now()::timestamp,now()::timestamp),
    ('00000000-0000-0000-0000-000000000002',1000000001,221,2,1,false,false,    'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,99,99,now()::timestamp,now()::timestamp),

    ('00000000-0000-0000-0000-000000000003',1000000002,222,2,1,false,true,    'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,99,99,now()::timestamp,now()::timestamp),
    ('00000000-0000-0000-0000-000000000004',1000000003,223,3,2,true,true,     'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,99,99,now()::timestamp,now()::timestamp),
    ('00000000-0000-0000-0000-000000000005',1000000004,222,3,2,true,true,     'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,99,99,now()::timestamp,now()::timestamp),
    
    ('00000000-0000-0000-0000-000000000006',1000000005,356,2,2,false,true,   'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,99,99,now()::timestamp,now()::timestamp);

INSERT INTO contract_amendment(
    uuid,
    id,
    status_id,
    pricing_expert_id,
    pricing_organization_unit_id,
    is_check_documentation,
    commission_date,
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
    commission_kind_id,
    purchasing_type_id
) values
    ('00000000-0000-0000-0001-000000000000',4000000001,221,2,1,false,null,'Subject',1,1,1,1,'Resume',1,now()::timestamp,now()::timestamp,99,99,1,1),
    ('00000000-0000-0000-0002-000000000000',4000000002,222,2,1,false,null,'Subject',1,1,1,1,'Resume',1,now()::timestamp,now()::timestamp,99,99,1,1),
    ('00000000-0000-0000-0003-000000000000',4000000003,223,3,2,false,null,'Subject',1,1,1,1,'Resume',1,now()::timestamp,now()::timestamp,99,99,1,1),
    ('00000000-0000-0000-0004-000000000000',4000000004,222,3,2,true,null,'Subject',1,1,1,1,'Resume',1,now()::timestamp,now()::timestamp,99,99,1,1),
    
    ('00000000-0000-0000-0005-000000000000',4000000005,356,3,2,false,null,'Subject',1,1,1,1,'Resume',1,now()::timestamp,now()::timestamp,99,99,1,1);
