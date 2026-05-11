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
    changed_at) values
        ('00000000-0000-0000-0000-000000000001','1','Слишком много комаров',1,2,3,4,5,6,10,2,251,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000002','2','Слишком много комаров',1,2,3,4,5,6,10,2,0,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000003','3','Слишком много комаров',1,2,3,4,5,6,10,2,223,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000004','4','Трубы должны блестеть',1,2,3,4,5,6,10,2,251,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000005','5','Трубы должны блестеть',1,2,3,4,5,6,10,2,251,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000006','6','Трубы должны блестеть',1,2,3,4,5,6,10,7,251,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000007','7','Трубы должны блестеть',1,2,3,4,5,6,10,2,251,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000008','8','Трубы должны блестеть',1,2,3,4,5,6,10,2,251,now()::date,now()::date,99,99,2,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000009','9','Трубы должны блестеть',1,2,3,4,5,6,10,2,223,now()::date,now()::date,99,99,1,now()::date,now()::date),
        ('00000000-0000-0000-0000-000000000010','10','Трубы должны блестеть',1,2,3,4,5,6,10,7,225,now()::date,now()::date,99,99,1,now()::date,now()::date);

INSERT INTO contract_amendment(
    uuid,
    id,
    status_id,
    contract_subject,
    customer_id,
    supplier_id,sum_excluded_vat,
    currency_id,
    pricing_expert_id, 
    pricing_resume,
    section_id,
    created_at,
    changed_at,
    created_by,
    changed_by,
    commission_kind_id,
    purchasing_type_id) values
                ('00000000-0000-0000-0001-000000000000',101,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
                ('00000000-0000-0000-0002-000000000000',102,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
                ('00000000-0000-0000-0003-000000000000',103,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
                ('00000000-0000-0000-0004-000000000000',104,225,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
                ('00000000-0000-0000-0005-000000000000',105,252,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
                ('00000000-0000-0000-0006-000000000000',106,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
                ('00000000-0000-0000-0007-000000000000',107,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
                
                ('00000000-0000-0000-0008-000000000000',108,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1),
                ('00000000-0000-0000-0009-000000000000',109,251,'Subject', 1, 1, 1, 1, 1, 'Resume', 1, now()::timestamp,now()::timestamp,99,99, 1, 1);

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
                ('00000000-0000-0000-0000-000000000002','2',1,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','1910-01-01'),
                ('00000000-0000-0000-0000-000000000003','3',1,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','1910-01-01'),
                    
                ('00000000-0000-0000-0000-000000000004','4',1,251,100,1,false,false,99,99,'1900-01-01','1900-01-01','1910-01-01');

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
            ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000004',2,101,false,false,'1900-01-02','1900-01-01',99,99),
            ('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000003','00000000-0000-0000-0006-000000000000',2,102,false,false,'1900-01-04','1900-01-01',99,99),
            
            ('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0008-000000000000',3,102,false,false,'1900-01-04','1900-01-01',99,99),
            ('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0009-000000000000',3,102,false,false,'1900-01-04','1900-01-01',99,99);

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
        ('00000000-0000-0000-0000-000000000001',1,'1900-01-01',100,2,true,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000002',2,'1900-01-02',100,2,false,'2000-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000003',3,'1900-01-03',200,2,false,'1900-01-01','1900-01-01',99,99),
        
        ('00000000-0000-0000-0000-000000000004',4,'1900-01-03',200,2,false,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000005',5,'1900-01-03',200,2,false,'1900-01-01','1900-01-01',99,99);

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
    changed_by,
    reviewed_at) values
        ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000007','1',false,false,false,'1900-01-01','1900-01-01',99,99,'1900-01-01'),
        ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000008','1',true,false,false,'1900-01-01','1900-01-01',99,99,'1900-01-01'),
        ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000007','1',false,false,false,'1901-01-01','1900-01-01',99,99,'1900-01-01'),
        ('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000008','1',true,false,false,'1901-01-01','1900-01-01',99,99,'1900-01-01'),
        ('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000007','1',false,false,false,'1900-01-01','1900-01-01',99,99,'1900-01-01'),
        ('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000003','00000000-0000-0000-0002-000000000000','1',true,false,false,'1901-01-01','1900-01-01',99,99,'1900-01-01'),
        
        ('00000000-0000-0000-0000-000000000007','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0008-000000000000','1',false,false,false,'1901-01-01','1900-01-01',99,99,'1900-01-01'),
        ('00000000-0000-0000-0000-000000000008','00000000-0000-0000-0000-000000000005','00000000-0000-0000-0008-000000000000','1',false,false,false,'1901-01-01','1900-01-01',99,99,'1900-01-01'),
        ('00000000-0000-0000-0000-000000000009','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0009-000000000000','1',false,false,false,'1901-01-01','1900-01-01',99,99,'1900-01-01');

INSERT INTO item_relation_agenda_protocol(agenda_item_uuid, agenda_uuid, protocol_uuid, protocol_item_uuid, created_at, created_by) values
        ('00000000-0000-0000-0000-000000000007','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000005',now()::timestamp,99),
        ('00000000-0000-0000-0000-000000000008','00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000005',now()::timestamp,99),
        ('00000000-0000-0000-0000-000000000009','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000006',now()::timestamp,99);

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
