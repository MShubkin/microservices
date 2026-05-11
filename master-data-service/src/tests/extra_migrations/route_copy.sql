INSERT INTO route_list (type_id, uuid, name_short, is_active, is_removed, created_at, changed_at, created_by, changed_by) VALUES
    (2, '00000000-0000-0000-0000-000000000001', 'name_short', true, false, now()::timestamp, now()::timestamp, 1, 1);

insert into route_data (route_uuid, data, created_at, changed_at, created_by, changed_by) values
    ('00000000-0000-0000-0000-000000000001', '{"assign_department": [{ "department_id": 1, "division": {"id": 1, "level": 1}}]}', now(), now(), 0, 0);

INSERT INTO route_crit (route_uuid, field_name, predicate, is_removed, created_at, created_by, changed_at, changed_by)
VALUES
	 ('00000000-0000-0000-0000-000000000001','pricing_organization_unit_id','{"kind" : "equal", "value" : 1}',false,'2025-04-03 20:46:13.13684+03',0,'2025-04-03 20:46:13.13684+03',0),
	 ('00000000-0000-0000-0000-000000000001','customer_id','{"kind" : "or", "predicates" : [{"kind" : "equal", "value" : 1000},{"kind" : "equal", "value" : 1500}]}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0),
	 ('00000000-0000-0000-0000-000000000001','section_id','{"kind" : "not_equal", "value" : 600}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0),
	 ('00000000-0000-0000-0000-000000000001','sum_excluded_vat','{"kind" : "any"}',false,'2025-04-03 21:33:25.316804+03',0,'2025-04-03 21:33:25.316804+03',0);
