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
        ('00000000-0000-0000-0000-000000000001',1,'Слишком много комаров',1,2,5,4,5,6,10,7,140,now()::date,now()::date,99,99,1,now()::date,now()::date, 1);


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
        ('00000000-0000-0000-0001-000000000000',101,'InPerson registered',7,3,4,6,150,99,99,now()::date,now()::date, 1, 1, 'Resume', 1, 1, 1, 1);

INSERT INTO attachment (
  uuid,
  object_uuid,
  category_id,
  number,
  parent_number,
  size,
  created_at,
  changed_at,
  created_by,
  changed_by,
  is_removed
) VALUES
    ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001',1,5,1,54,'1999-1-1','1999-1-1',1,1, false),
    ('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000001',2,4,1,3453,'1999-1-1','1999-1-1',1,1, false),
    ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000001',3,4,1,3453,'1999-1-1','1999-1-1',1,1, true),
    ('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0001-000000000000',4,3,1,345345,'1999-1-1','1999-1-1',1,1, false),
    ('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0001-000000000000',5,2,1,100,'1999-1-1','1999-1-1',1,1, false),
    ('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0001-000000000000',6,4,1,3453,'1999-1-1','1999-1-1',1,1, true);
