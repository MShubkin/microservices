INSERT INTO plan(
    uuid,
    id,
    status_id,
    pricing_expert_id, 
    pricing_resume,
    pricing_method_id,
    expert_conclusion_id,
    savings_accounting_id,
    is_check_documentation,

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
        -- Невалидная ППЗ
        ('00000000-0000-0000-0000-000000000001',1,222, NULL, NULL, 0, NULL, 0, false,     'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp),

        ('00000000-0000-0000-0000-000000000002',2,222, 1, 'Resume', 1, 1, 1, false,      'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp),
        -- Невалидная ППЗ по заключения Эксперта АЦ
        ('00000000-0000-0000-0000-000000000003',3,222, 1, 'Resume', 1, 5, 1, true,     'Слишком много комаров',1,2,3,4,5,6,10,2,'1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp);

INSERT INTO contract_amendment(
    uuid,
    id,
    status_id,
    pricing_expert_id, 
    pricing_resume,
    pricing_method_id,
    expert_conclusion_id,
    savings_accounting_id,

    contract_subject,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    currency_id,
    section_id,
    created_at,
    changed_at,
    created_by,
    changed_by,
    commission_kind_id,
    purchasing_type_id) values
        -- Невалидная ДС, так как все некоторые поля не заполнены
        ('00000000-0000-0000-0001-000000000000',11,342, 1, 'Resume', 1, 1, 0,    'Subject', 1, 1, 1, 1,  1, now()::timestamp,now()::timestamp,99,99, 1, 1),
        ('00000000-0000-0000-0002-000000000000',12,342, 1, 'Resume', 1, 1, 1,    'Subject', 1, 1, 1, 1,  1, now()::timestamp,now()::timestamp,99,99, 1, 1),
        ('00000000-0000-0000-0003-000000000000',13,352, 1, 'Resume', 1, 4, 1,    'Subject', 1, 1, 1, 1,  1, now()::timestamp,now()::timestamp,99,99, 1, 1);
