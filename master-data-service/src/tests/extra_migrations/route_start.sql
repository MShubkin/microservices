INSERT INTO route_list (type_id, uuid, is_active, created_at, changed_at, created_by, changed_by) VALUES
        (1, '00000000-0000-0000-0000-000000000001', false, now()::timestamp, now()::timestamp, 1, 1),
        (1, '00000000-0000-0000-0000-000000000002', true, now()::timestamp, now()::timestamp, 1, 1),
        (1, '00000000-0000-0000-0000-000000000003', false, now()::timestamp, now()::timestamp, 1, 1),
        (1, '00000000-0000-0000-0000-000000000004', false, now()::timestamp, now()::timestamp, 1, 1),
        (1, '00000000-0000-0000-0000-000000000005', false, now()::timestamp, now()::timestamp, 1, 1),
        (1, '00000000-0000-0000-0000-000000000006', true, now()::timestamp, now()::timestamp, 1, 1),
        (1, '00000000-0000-0000-0000-000000000007', false, now()::timestamp, now()::timestamp, 1, 1),
        (1, '00000000-0000-0000-0000-000000000008', true, now()::timestamp, now()::timestamp, 1, 1);

INSERT INTO route_data (route_uuid, data, created_at, changed_at, created_by, changed_by)
VALUES
    ('00000000-0000-0000-0000-000000000001', '{"assign_department": [{"department_id": 1, "division": {"id": 1, "level": 3}}]}', now(), now(), 1, 1),
    ('00000000-0000-0000-0000-000000000002', '{"assign_department": [{"department_id": 2, "division": {"id": 2, "level": 3}}]}', now(), now(), 1, 1),
    ('00000000-0000-0000-0000-000000000003', '{"assign_department": [{"department_id": 1, "division": {"id": 1, "level": 3}}]}', now(), now(), 1, 1),
    ('00000000-0000-0000-0000-000000000004', '{"assign_department": [{"department_id": 1, "division": {"id": 1, "level": 3}}]}', now(), now(), 1, 1),
    ('00000000-0000-0000-0000-000000000005', '{"assign_department": [{"department_id": 2, "division": {"id": 2, "level": 3}}]}', now(), now(), 1, 1),
    ('00000000-0000-0000-0000-000000000006', '{"assign_department": [{"department_id": 1, "division": {"id": 1, "level": 3}}]}', now(), now(), 1, 1),
    ('00000000-0000-0000-0000-000000000007', '{"assign_department": [{"department_id": 1, "division": {"id": 1, "level": 3}}]}', now(), now(), 1, 1),
    ('00000000-0000-0000-0000-000000000008', '{"assign_department": [{"department_id": 1, "division": {"id": 1, "level": 3}}]}', now(), now(), 1, 1);


INSERT INTO route_crit (route_uuid, field_name, predicate, created_at, changed_at, created_by, changed_by) VALUES
('00000000-0000-0000-0000-000000000001', 'customer_id', '{"kind": "or", "predicates": [{"kind": "equal", "value": 53}, {"kind": "equal", "value": 99}]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000001', 'section_id', '{"kind": "equal", "value": "goats"}', now()::timestamp, now()::timestamp, 1, 1),

('00000000-0000-0000-0000-000000000002', 'customer_id', '{"kind": "equal", "value": 12343}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000002', 'section_id', '{"kind": "not_equal", "value": "12"}', now()::timestamp, now()::timestamp, 1, 1),

('00000000-0000-0000-0000-000000000003', 'customer_id', '{"kind": "equal", "value": true}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000003', 'section_id', '{"kind": "not_equal", "value": false}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000003', 'sum_excluded_vat', '{"kind": "or", "predicates": [{"kind": "less", "value": 44}, {"kind": "less_equal", "value": "hamster"}]}', now()::timestamp, now()::timestamp, 1, 1),

('00000000-0000-0000-0000-000000000004', 'customer_id', '{"kind": "equal", "value": false}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000004', 'section_id', '{"kind": "not_equal", "value": true}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000004', 'sum_excluded_vat', '{"kind": "or", "predicates": [{"kind": "less", "value": 777}, {"kind": "less_equal", "value": "chaos"}]}', now()::timestamp, now()::timestamp, 1, 1),

('00000000-0000-0000-0000-000000000005', 'customer_id', '{"kind": "equal", "value": true}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000005', 'section_id', '{"kind": "not_equal", "value": false}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000005', 'sum_excluded_vat', '{"kind": "or", "predicates": [{"kind": "less", "value": 44}, {"kind": "less_equal", "value": "hamster"}]}', now()::timestamp, now()::timestamp, 1, 1),

('00000000-0000-0000-0000-000000000006', 'customer_id', '{"kind": "equal", "value": true}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000006', 'section_id', '{"kind": "not_equal", "value": false}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000006', 'sum_excluded_vat', '{"kind": "or", "predicates": [{"kind": "less", "value": 44}, {"kind": "less_equal", "value": "hamster"}]}', now()::timestamp, now()::timestamp, 1, 1),

('00000000-0000-0000-0000-000000000007', 'budget_item_id', '{"kind": "in_tree","roots": [1,2,3,4,5], "dictionary": "budget_item"}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000007', 'category_id', '{"kind": "in_tree","roots": [1,2,3,4,5], "dictionary": "category"}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000007', 'contract_amendment_kind_id', '{"kind": "in", "values": [1,2,3,4,5]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000007', 'customer_id', '{"kind": "in", "values": [1,2,3,4,5]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000007', 'funding_source_id', '{"kind": "in", "values": [1,2,3,4,5]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000007', 'object_type_id', '{"kind": "in", "values": [1,2,3,4,5]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000007', 'okato_id', '{"kind": "in", "values": [1,2,3,4,5]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000007', 'purchasing_method_id', '{"kind": "in", "values": [1,2,3,4,5]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000007', 'section_id', '{"kind": "in", "values": [1,2,3,4,5]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000007', 'sum_excluded_vat', '{"kind": "or", "predicates": [{"kind": "less", "value": 44}, {"kind": "less_equal", "value": "hamster"}]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000007', 'okpd2_id', '{"kind": "in_tree","roots": [1,2,3,4,5], "dictionary": "okpd2"}', now()::timestamp, now()::timestamp, 1, 1),

('00000000-0000-0000-0000-000000000008', 'budget_item_id', '{"kind": "in_tree","roots": [5,4,3,2,1], "dictionary": "budget_item"}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000008', 'category_id', '{"kind": "in_tree","roots": [3,1,2,5,4], "dictionary": "category"}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000008', 'contract_amendment_kind_id', '{"kind": "in", "values": [3,1,2,5,4]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000008', 'customer_id', '{"kind": "in", "values": [5,4,3,2,1]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000008', 'funding_source_id', '{"kind": "in", "values": [3,1,2,4,5]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000008', 'object_type_id', '{"kind": "in", "values": [3,5,4,2,1]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000008', 'okato_id', '{"kind": "in", "values": [4,5,1,2,3]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000008', 'purchasing_method_id', '{"kind": "in", "values": [4,1,2,3,5]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000008', 'section_id', '{"kind": "in", "values": [5,4,3,2,1]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000008', 'sum_excluded_vat', '{"kind": "or", "predicates": [{"kind": "less", "value": 44}, {"kind": "less_equal", "value": "hamster"}]}', now()::timestamp, now()::timestamp, 1, 1),
('00000000-0000-0000-0000-000000000008', 'okpd2_id', '{"kind": "in_tree","roots": [5,1,2,3,4], "dictionary": "okpd2"}', now()::timestamp, now()::timestamp, 1, 1);
