INSERT INTO route_list (type_id, uuid, is_active, is_removed, created_at, changed_at, created_by, changed_by) VALUES
    (1, '00000000-0000-0000-0000-000000000001', true, false, now()::timestamp, now()::timestamp, 1, 1),
    (1, '00000000-0000-0000-0000-000000000002', false, false, now()::timestamp, now()::timestamp, 1, 1),
    (1, '00000000-0000-0000-0000-000000000003', false, false, now()::timestamp, now()::timestamp, 1, 1),
    (1, '00000000-0000-0000-0000-000000000004', false, true, now()::timestamp, now()::timestamp, 1, 1);
