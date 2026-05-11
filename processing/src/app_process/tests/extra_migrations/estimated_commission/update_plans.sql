INSERT INTO plan(
    uuid,
    id,
    commission_kind_id,
    purchasing_type_id,
    status_id,
    year,
    is_check_documentation,
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
    changed_at,
    is_priority_project
    ) values
        ('00000000-0000-0000-0000-000000000001', 1,  1, 2, 221, date_part('year', now()), false, 1,    2,3,4,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,false),
        ('00000000-0000-0000-0000-000000000002', 2,  1, 2, 221, 0,                        false, 1,    2,3,4,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,false);

INSERT INTO contract_amendment(
    uuid,
    id,
    commission_kind_id,
    purchasing_type_id,
    status_id,
    year,
    is_check_documentation,
    pricing_expert_id,

    contract_subject,customer_id,
    supplier_id,sum_excluded_vat,
    currency_id,  
    pricing_resume, section_id,
    created_at,
    changed_at,
    created_by,
    changed_by
    ) values
        ('00000000-0000-0000-0001-000000000000', 1,  1, 2, 221, date_part('year', now()), true,  1,    'Subject', 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99),
        ('00000000-0000-0000-0002-000000000000', 2,  1, 2, 221, 0,                        false, 1,    'Subject', 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99);

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
