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
    created_by,
    changed_by,
    pricing_organization_unit_id,
    created_at,
    changed_at,
    pricing_sum_excluded_vat) values
        ('00000000-0000-0000-0000-000000000001',1,'Слишком много комаров',1,2,3,4,5,6,10,7,0,now()::date,now()::date,99,99,1,now()::date,now()::date, 1),
        ('00000000-0000-0000-0000-000000000002',2,'Слишком много комаров',1,2,3,4,5,6,10,7,0,now()::date,now()::date,99,99,1,now()::date,now()::date, 1),
        -- не в agenda_item
        ('00000000-0000-0000-0000-000000000003',3,'Слишком много комаров',1,2,3,4,5,6,10,7,0,now()::date,now()::date,99,99,1,now()::date,now()::date, 1);


INSERT INTO contract_amendment(
    uuid,
    id,
    contract_subject,
    customer_id,
    supplier_id,
    sum_excluded_vat,
    currency_id,
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
    pricing_sum_excluded_vat) values
        ('00000000-0000-0000-0000-000000000011',11,'InPerson registered',2,3,4,6,251,99,99,now()::date,now()::date, 1, 1, 'Resume', 1, 1, 1, 1),
        ('00000000-0000-0000-0000-000000000012',12,'InPerson registered',2,3,4,6,251,99,99,now()::date,now()::date, 1, 1, 'Resume', 1, 1, 1, 1);


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
        ('00000000-0000-0000-0000-000000000001','1',1,253,100,1,false,false,99,99,'1900-01-01','1900-01-01','1910-01-01');

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
        ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001',2,102,false,false,'1900-01-02','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000011',2,103,false,false,'1900-01-03','1900-01-01',99,99);

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
        ('00000000-0000-0000-0000-000000000001',1,'2000-01-01',100,2,false,'1900-01-01','1900-01-01',99,99);

INSERT INTO agenda_item(
    uuid,
    agenda_uuid,
    source_uuid,
    number,
    is_registered_by_d647,
    is_removed,
    is_excluded,
    created_at,
    changed_at,
    created_by,
    changed_by) values
        ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','1',false,false,false,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000011','1',false,false,false,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000002','1',false,false,true,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000012','1',false,false,false,'1900-01-01','1900-01-01',99,99);

INSERT INTO item_relation_agenda_protocol(
    agenda_item_uuid,
    agenda_uuid,
    protocol_item_uuid,
    protocol_uuid,
    created_at,
    created_by
) VALUES 
        ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', now(), 1),
        ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', now(), 1); 
