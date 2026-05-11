INSERT INTO protocol(
    uuid,
    id,
    protocol_type_id,
    protocol_date,
    status_id,
    pricing_organization_unit_id,
    created_at,
    changed_at,
    created_by,
    changed_by) values
        ('00000000-0000-0000-0000-000000000001',1,1,'2000-01-01',100,1,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000002',2,2,'2000-01-01',100,1,'2000-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000003',3,1,'2000-01-01',100,1,'2000-01-01','1900-01-01',99,99);

INSERT INTO protocol_item(
    uuid,
    protocol_uuid,
    source_uuid,
    number,
    is_registered_by_d647,
    is_removed,
    is_excluded,
    created_at,
    changed_at,
    created_by,
    changed_by) values
        ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000005','1',false,true,true,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000005','1',false,false,false,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000003','00000000-0000-0000-0005-000000000000','1',false,false,false,'1900-01-01','1900-01-01',99,99);

-- Plans 5-8 do not belong to intramural.
INSERT INTO plan(
    uuid,
    id,
    contract_subject,
    commission_kind_id,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    pricing_sum_excluded_vat,
    sum_excluded_vat_rub,
    currency_id,
    currency_rate,
    purchasing_type_id,
    status_id,
    delivery_start_date,
    delivery_end_date,
    created_by,
    changed_by,
    pricing_organization_unit_id,
    created_at,
    changed_at) values
        ('00000000-0000-0000-0000-000000000001','1','InPerson registered',1,2,3,4,5,1,6,10,2,251,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000002','2','InPerson registered',1,2,3,4,5,1,6,10,2,251,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000003','3','InPerson unregistered',2,2,3,4,5,1,6,10,2,252,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000004','4','InPerson unregistered',2,2,3,4,5,1,6,10,2,252,now()::date,now()::date,99,99,1,now()::date,now()::date),

        ('00000000-0000-0000-0000-000000000005','5','Correspondence registered',2,2,3,4,5,1,6,10,2,252,now()::date,now()::date,99,99,1,now()::date,now()::date);

INSERT INTO contract_amendment(
    uuid,
    id,
    commission_kind_id,
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
    purchasing_type_id,
    delta_sum_excluded_vat,
    pricing_delta_sum_excluded_vat) values
        ('00000000-0000-0000-0001-000000000000',101,1,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 3, 4),
        ('00000000-0000-0000-0002-000000000000',102,1,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 3, 4),
        ('00000000-0000-0000-0003-000000000000',103,2,252,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 3, 4),
        ('00000000-0000-0000-0004-000000000000',104,2,252,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 3, 4),

        ('00000000-0000-0000-0005-000000000000',105,1,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 3, 4);
