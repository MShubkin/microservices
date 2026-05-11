INSERT INTO plan(
    uuid,
    id,
    status_id,
    is_check_documentation,
    pricing_expert_id,
    
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
    pricing_organization_unit_id,
    section_id,
    created_by,
    changed_by,
    created_at,
    changed_at) values
        ('00000000-0000-0000-0000-000000000001',1,222,true,123,    'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp),
        ('00000000-0000-0000-0000-000000000002',2,222,true,123,    'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp),
        ('00000000-0000-0000-0000-000000000003',3,352,true,125,     'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp),
        ('00000000-0000-0000-0000-000000000004',4,221,true,123,     'Трубы должны блестеть',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp),
        
        ('00000000-0000-0000-0000-000000000101',101,221,false,null,     'Трубы должны блестеть',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp);

INSERT INTO contract_amendment(
    uuid,
    id,
    status_id,
    is_check_documentation,
    pricing_expert_id,

    contract_subject,customer_id,
    supplier_id,sum_excluded_vat,
    currency_id,  
    pricing_resume, section_id,
    created_at,
    changed_at,
    created_by,
    changed_by,
    commission_kind_id,
    purchasing_type_id) values
        ('00000000-0000-0000-0001-000000000000',11,222,true,123,  'Subject', 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
        ('00000000-0000-0000-0002-000000000000',12,222,true,125,  'Subject', 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
        ('00000000-0000-0000-0003-000000000000',13,221,true,123,   'Subject', 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
        ('00000000-0000-0000-0004-000000000000',14,221,true,null,  'Subject', 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1);
