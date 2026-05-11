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
    sum_excluded_vat, sum_excluded_vat_rub,
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
    expert_conclusion_id)
VALUES
('00000000-0000-0000-0000-000000000001',1,1,2,221,date_part('year', now()),1,2,3,4,5,6,10,'ППЗ','1901-01-01','1901-01-01',1,1,99,99,now()::timestamp,now()::timestamp,2);

ALTER TABLE plan_version DROP COLUMN pricing_version;
ALTER TABLE plan_version ADD COLUMN pricing_version SMALLINT NOT NULL;
INSERT INTO plan_version (SELECT *,1 FROM plan);

INSERT INTO contract_amendment(
    uuid,
    id,
    status_id,
    contract_subject,
    is_actual,
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
    commission_date,
    commission_kind_id,
    purchasing_type_id,
    pricing_organization_unit_id,
    expert_conclusion_id)
VALUES
('00000000-0000-0000-0001-000000000001',4002,251,'ДС',true,12,22,1,1,1,'Resume',1,now()::timestamp,now()::timestamp,99,99,now(),1,1,1,3);

ALTER TABLE contract_amendment_version DROP COLUMN pricing_version;
ALTER TABLE contract_amendment_version ADD COLUMN pricing_version SMALLINT NOT NULL;
INSERT INTO contract_amendment_version (SELECT *,1 FROM contract_amendment);

INSERT INTO status_history (uuid, object_uuid, status_id, comment, created_at, created_by)
VALUES
('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000001',222,'ppz status 222','2021-09-30 11:21:12.877345',123),
('00000000-0000-0000-0000-000000000002','00000000-0000-0000-0000-000000000001',342,'ppz status 342','2022-09-30 12:22:26.912647',123),
('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000001',352,'ppz status 352','2023-09-30 13:23:26.962647',123),
('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000001',223,'ppz status 223','2023-11-30 13:23:26.962647',123),
('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000001',225,'ppz status 225','2024-11-30 14:23:26.962647',123),

('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0001-000000000001',222,'dc status 222','2024-12-01 06:43:09.963269',123),
('00000000-0000-0000-0000-000000000007','00000000-0000-0000-0001-000000000001',343,'dc status 343','2024-12-02 07:43:09.963269',123),
('00000000-0000-0000-0000-000000000008','00000000-0000-0000-0001-000000000001',353,'dc status 353','2024-12-22 08:43:09.961249',123);
