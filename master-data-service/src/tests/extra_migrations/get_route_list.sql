INSERT INTO route_list (type_id, uuid, is_active, is_removed, created_at, changed_at, created_by, changed_by) VALUES
    -- assign_expert
    -- pricing_organization_unit_id = 1
    (2, '00000000-0000-0000-0000-000000000001', true, false, now()::timestamp, now()::timestamp, 1, 1),
    (2, '00000000-0000-0000-0000-000000000002', false, false, now()::timestamp, now()::timestamp, 1, 1),
    (2, '00000000-0000-0000-0000-000000000003', false, false, now()::timestamp, now()::timestamp, 1, 1),
    (2, '00000000-0000-0000-0000-000000000004', false, true, now()::timestamp, now()::timestamp, 1, 1),
    -- pricing_organization_unit_id = 2
    (2, '00000000-0000-0000-0000-000000000005', false, false, now()::timestamp, now()::timestamp, 1, 1),
    -- assign_department
    (1, '00000000-0000-0000-0001-000000000001', true, false, now()::timestamp, now()::timestamp, 1, 1),
    (1, '00000000-0000-0000-0001-000000000002', true, false, now()::timestamp, now()::timestamp, 1, 1);

insert into route_data (route_uuid, data, created_at, changed_at, created_by, changed_by) values
    ('00000000-0000-0000-0000-000000000001', '{"assign_department": [{ "department_id": 1, "division": {"id": 1, "level": 1}}]}', now(), now(), 0, 0),
    ('00000000-0000-0000-0000-000000000002', '{"assign_department": [{ "department_id": 2, "division": {"id": 2, "level": 2}}]}', now(), now(), 0, 0),
    ('00000000-0000-0000-0000-000000000003', '{"assign_department": [{ "department_id": 3, "division": {"id": 3, "level": 3}}]}', now(), now(), 0, 0),
    ('00000000-0000-0000-0000-000000000004', '{"assign_department": [{ "department_id": 4, "division": {"id": 4, "level": 1}}]}', now(), now(), 0, 0),
    ('00000000-0000-0000-0000-000000000005', '{"assign_department": [{ "department_id": 5, "division": {"id": 5, "level": 2}}]}', now(), now(), 0, 0),
    ('00000000-0000-0000-0001-000000000001', '{"assign_department": [{ "department_id": 10, "division": {"id": 100, "level": 1}}]}', now(), now(), 0, 0),
    ('00000000-0000-0000-0001-000000000002', '{"assign_department": [{ "department_id": 20, "division": {"id": 201, "level": 2}}]}', now(), now(), 0, 0);

INSERT INTO route_crit (route_uuid, field_name, predicate, is_removed, created_at, created_by, changed_at, changed_by)
VALUES
	 ('00000000-0000-0000-0000-000000000001','pricing_organization_unit_id','{"kind" : "equal", "value" : 1}',false,'2025-04-03 20:46:13.13684+03',0,'2025-04-03 20:46:13.13684+03',0),
	 ('00000000-0000-0000-0000-000000000002','pricing_organization_unit_id','{"kind" : "equal", "value" : 1}',false,'2025-04-03 20:46:13.13684+03',0,'2025-04-03 20:46:13.13684+03',0),
	 ('00000000-0000-0000-0000-000000000003','pricing_organization_unit_id','{"kind" : "equal", "value" : 1}',false,'2025-04-03 20:46:13.13684+03',0,'2025-04-03 20:46:13.13684+03',0),
	 ('00000000-0000-0000-0000-000000000004','pricing_organization_unit_id','{"kind" : "equal", "value" : 1}',false,'2025-04-03 20:46:13.13684+03',0,'2025-04-03 20:46:13.13684+03',0),
	 ('00000000-0000-0000-0000-000000000005','pricing_organization_unit_id','{"kind" : "equal", "value" : 2}',false,'2025-04-03 20:46:13.13684+03',0,'2025-04-03 20:46:13.13684+03',0),
	 ('00000000-0000-0000-0000-000000000001','object_type_id','{"kind": "equal", "value": 1}',false,'2025-04-03 20:46:13.13684+03',0,'2025-04-03 20:46:13.13684+03',0),
	 ('00000000-0000-0000-0000-000000000002','object_type_id','{"kind": "equal", "value": 2}',false,'2025-04-03 20:46:13.13684+03',0,'2025-04-03 20:46:13.13684+03',0),
	 ('00000000-0000-0000-0000-000000000003','object_type_id','{"kind": "in", "values": [1, 2]}',false,'2025-04-03 20:46:13.13684+03',0,'2025-04-03 20:46:13.13684+03',0),
	 ('00000000-0000-0000-0000-000000000001','customer_id','{"kind" : "or", "predicates" : [{"kind" : "equal", "value" : 1000},{"kind" : "equal", "value" : 1500}]}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0),
	 ('00000000-0000-0000-0000-000000000001','section_id','{"kind" : "not_equal", "value" : 600}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0),
	 ('00000000-0000-0000-0000-000000000001','sum_excluded_vat','{"kind" : "any"}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0),
	 ('00000000-0000-0000-0000-000000000001','purchasing_type_id','{"kind" : "not_equal", "value" : 500}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0),
	 ('00000000-0000-0000-0000-000000000001','okdp2','{"kind" : "or", "predicates" : [{"kind" : "less", "value" : 900},{"kind" : "less", "value" : 950}]}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0),
	 ('00000000-0000-0000-0000-000000000001','budget_item_id','{"kind" : "or", "predicates" : [{"kind" : "equal", "value" : 800},{"kind" : "equal", "value" : 700}]}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0),
	 ('00000000-0000-0000-0000-000000000002','section_id','{"kind" : "or", "predicates" : [{"kind" : "less_equal", "value" : 500},{"kind" : "greater_equal", "value" : 1500}]}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0),
	 ('00000000-0000-0000-0000-000000000003','okdp2','{"kind" : "or", "predicates" : [{"kind" : "less", "value" : 1200},{"kind" : "less", "value" : 2500}]}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0);
