INSERT INTO route_list (type_id, uuid, is_active, is_removed, created_at, changed_at, created_by, changed_by) VALUES
    --pricing_organization_unit_id = 1
    (2, '00000000-0000-0000-0000-000000000001', true, false, now()::timestamp, now()::timestamp, 1, 1),
    (2, '00000000-0000-0000-0000-000000000002', false, false, now()::timestamp, now()::timestamp, 1, 1),
    (2, '00000000-0000-0000-0000-000000000003', false, false, now()::timestamp, now()::timestamp, 1, 1),
    (2, '00000000-0000-0000-0000-000000000004', false, true, now()::timestamp, now()::timestamp, 1, 1),
    --pricing_organization_unit_id = 2
    (2, '00000000-0000-0000-0000-000000000005', false, false, now()::timestamp, now()::timestamp, 1, 1),
    -- specialized deps
    (1, '00000000-0000-0000-0000-000000000006', true, false, now()::timestamp, now()::timestamp, 1, 1),
    (1, '00000000-0000-0000-0000-000000000007', true, false, now()::timestamp, now()::timestamp, 1, 1);

insert into route_data (route_uuid, data, created_at, changed_at, created_by, changed_by)
values
    ('00000000-0000-0000-0000-000000000001', '{"assign_expert": {"primary_pricing_expert_list": [{"expert_id": 580, "date_range": ["19.05.2025", "28.05.2025"]}], "replacement_pricing_expert_list": [{"expert_id": 1085, "date_range": ["01.05.2025", "06.05.2025"]}]}}', now(), now(), 1, 1),
    ('00000000-0000-0000-0000-000000000002', '{"assign_expert": {"primary_pricing_expert_list": [{"expert_id": 0, "date_range": ["07.04.2025", "11.04.2025"]}], "replacement_pricing_expert_list": [{"expert_id": 1814, "date_range": ["14.04.2025", "18.04.2025"]}]}}', now(), now(), 1, 1),
    ('00000000-0000-0000-0000-000000000006', '{"assign_department": [{"department_id": 2, "division": {"id": 2, "level": 3}}]}', now(), now(), 1, 1),
    ('00000000-0000-0000-0000-000000000007', '{"assign_department": [{"department_id": 4, "division": {"id": 5, "level": 3}}]}', now(), now(), 1, 1);

INSERT INTO public.route_crit (route_uuid,field_name,predicate,is_removed,created_at,created_by,changed_at,changed_by) VALUES
	 ('00000000-0000-0000-0000-000000000001','pricing_organization_unit_id','{"kind" : "equal", "value" : 1}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000002','pricing_organization_unit_id','{"kind" : "equal", "value" : 1}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000003','pricing_organization_unit_id','{"kind" : "equal", "value" : 1}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000004','pricing_organization_unit_id','{"kind" : "equal", "value" : 1}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000005','pricing_organization_unit_id','{"kind" : "equal", "value" : 2}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000001','customer_id','{"kind" : "or", "predicates" : [{"kind" : "equal", "value" : 1500},{"kind" : "equal", "value" : 1000}]}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000001','section_id','{"kind" : "not_equal", "value" : 600}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000001','sum_excluded_vat','{"kind" : "any"}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000001','purchasing_type_id','{"kind" : "not_equal", "value" : 500}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000001','okdp2','{"kind" : "in_tree", "dictionary": "okpd2", "roots": [128]}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000001','budget_item_id','{"kind" : "or", "predicates" : [{"kind" : "equal", "value" : 700},{"kind" : "equal", "value" : 800}]}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000003','okdp2','{"kind" : "in_tree", "dictionary": "okpd2", "roots": [345]}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0),
	 ('00000000-0000-0000-0000-000000000006','section_id','{"kind" : "none"}',false,'2025-03-31 08:11:14.704457',0,'2025-03-31 08:11:14.704457',0);
