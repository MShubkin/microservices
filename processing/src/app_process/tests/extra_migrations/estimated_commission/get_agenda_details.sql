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
        ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000011','1',false,true,false,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000002','1',true,false,true,'1900-01-01','1900-01-01',99,99),
        ('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000012','2',false,false,false,'1900-01-01','1900-01-01',99,99);

INSERT INTO estimated_commission_partner (
  uuid,
  protocol_agenda_uuid,
  user_id,
  role_id,
  is_removed,
  created_at,
  changed_at,
  created_by,
  changed_by
) VALUES
    ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001',1,2,false,'1999-1-1','1999-1-1',1,1),
    ('00000000-0000-0000-0000-000000000011','00000000-0000-0000-0000-000000000001',3,1,false,'1999-1-1','1999-1-1',1,1),
    ('00000000-0000-0000-0000-000000000012','00000000-0000-0000-0000-000000000001',1,1,false,'1999-1-1','1999-1-1',1,1),
    -- This user does not belong to agenda 1.
    ('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000001',2,1,true,'1999-1-1','1999-1-1',1,1),
    ('00000000-0000-0000-0000-000000000007','00000000-0000-0000-0000-000000000001',2,2,true,'1999-1-1','1999-1-1',1,1),
    -- This user does not belong to agenda 1.
    ('00000000-0000-0000-0000-000000000009','00000000-0000-0000-0000-000000000099',3,2,false,'1999-1-1','1999-1-1',1,1);

INSERT INTO partner_type_commission(
    uuid,
    user_id,
    role_id,
    protocol_type_id,
    created_at,
    changed_at,
    created_by,
    changed_by
) VALUES 
    ('00000000-0000-0000-0000-000000000001', 1, 111, 1, now(), now(), 0, 0),
    ('00000000-0000-0000-0000-000000000002', 2, 222, 2, now(), now(), 0, 0),
    ('00000000-0000-0000-0000-000000000003', 2, 333, 1, now(), now(), 0, 0),
    ('00000000-0000-0000-0000-000000000004', 3, 444, 2, now(), now(), 0, 0);

INSERT INTO attachment (
  uuid,
  object_uuid,
  "number",
  parent_number,
  "size",
  created_at,
  changed_at,
  created_by,
  changed_by
) VALUES
    ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001',5,1,54,'1999-1-1','1999-1-1',1,1),
    ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000001',4,1,3453,'1999-1-1','1999-1-1',1,1),
    ('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000001',3,1,345345,'1999-1-1','1999-1-1',1,1),
    ('00000000-0000-0000-0000-000000000009','00000000-0000-0000-0000-000000000099',2,1,100,'1999-1-1','1999-1-1',1,1);

INSERT INTO status_history (
  uuid,
  object_uuid,
  created_at,
  created_by
) VALUES
    ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001','1999-1-1',1),
    ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000099','1999-1-1',1),
    ('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000066','1999-1-1',1),
    ('00000000-0000-0000-0000-000000000007','00000000-0000-0000-0000-000000000022','1999-1-1',1);
