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
    ('00000000-0000-0000-0000-000000000003', 3, 333, 1, now(), now(), 0, 0),
    ('00000000-0000-0000-0000-000000000004', 4, 444, 2, now(), now(), 0, 0);
