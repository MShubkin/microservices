INSERT INTO plan(
    uuid,
    id,
    commission_kind_id,
    purchasing_type_id,
    status_id,
    year,
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
    changed_at
    ) values
        ('00000000-0000-0000-0000-000000000001', 1,  1, 2, 221, date_part('year', now()), 1,                       2,3,4,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp),
        ('00000000-0000-0000-0000-000000000002', 2,  1, 2, 221, 0,                        1,                       2,3,4,5,6,10,'Слишком много комаров','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp);
-- Version updates fail if there are no versions, so we create them manually.
ALTER TABLE plan_version DROP COLUMN pricing_version;
ALTER TABLE plan_version ADD COLUMN pricing_version SMALLINT NOT NULL;
INSERT INTO plan_version (SELECT *,1 FROM plan);

INSERT INTO public.plan_item(
    uuid,plan_uuid,category_id,description_internal,quantity,
    id,product_type_id,budget_item_id,okved2_id,unit_id,payment_balance_item_id,is_not_russian_delivery,created_by,changed_by,currency_id,currency_rate,
    created_at, changed_at
    )
    VALUES 
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001',1000,'труба, железная,',1000,0,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date),
    ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001',9000,'труба, железная,',9000,1,1,2,3,4,5,false,99,99,1,100,now()::date,now()::date);

INSERT INTO attachment(
  uuid,
  object_uuid,
  "number",
  parent_number,
  "size",
  created_at,
  changed_at,
  created_by,
  changed_by,
  category_id
) VALUES
    ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001',5,1,54,'1999-1-1','1999-1-1',1,1,1),
    ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000001',5,1,3453,'1999-1-1','1999-1-1',1,1,1),
    ('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000001',3,1,345345,'1999-1-1','1999-1-1',1,1,2),
    ('00000000-0000-0000-0000-000000000009','00000000-0000-0000-0000-000000000001',2,1,100,'1999-1-1','1999-1-1',1,1,2);
