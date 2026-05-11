INSERT INTO price_analysis_user (
    id,
    user_id,
    type_user_id,
    pricing_organization_unit_id,
    start_date,
    end_date,
    created_at,
    changed_at,
    created_by,
    changed_by
) values
    (1, 1, 1, 2, now()::timestamp - INTERVAL '1 hour', now()::timestamp + INTERVAL '1 hour', now()::timestamp, now()::timestamp, 1, 1),
    (2, 2, 2, 2, now()::timestamp - INTERVAL '1 hour', now()::timestamp + INTERVAL '1 hour', now()::timestamp, now()::timestamp, 1, 1),
    (3, 3, 2, 1, now()::timestamp - INTERVAL '1 hour', now()::timestamp + INTERVAL '1 hour', now()::timestamp, now()::timestamp, 1, 1),
    (4, 3, 3, 3, now()::timestamp - INTERVAL '1 hour', now()::timestamp + INTERVAL '1 hour', now()::timestamp, now()::timestamp, 1, 1),
    
    -- Не подходит по start_date, end_date
    (100, 3, 2, 1, now()::timestamp + INTERVAL '1 hour', now()::timestamp + INTERVAL '2 hour', now()::timestamp, now()::timestamp, 1, 1);
