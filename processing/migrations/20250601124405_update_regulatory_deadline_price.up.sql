/*Galko: Это версия без проверки существования данных */
/*
COPY public.regulatory_deadline_price (uuid, section, field_id, color_scheme_id, type_criticality, start_day, end_day, created_by, created_at, changed_by, changed_at, status) FROM stdin;
d6229360-06fc-0000-805c-566ff2f30001    10      19      1       1       1       5       662     2025-01-20 07:21:09.020604      662     2025-01-20 07:21:09.020604      f
d6229360-06fc-0000-805c-566ff2f30002    10      19      2       2       6       7       662     2025-01-20 07:21:09.020604      662     2025-01-20 07:21:09.020604      f
d6229360-06fc-0000-805c-566ff2f30003    10      19      3       3       8       9999    662     2025-01-20 07:21:09.020604      662     2025-01-20 07:21:09.020604      f
d6229360-06fc-0000-805c-566ff2f30004    7       19      1       1       1       5       662     2025-01-20 07:21:09.020604      662     2025-01-20 07:21:09.020604      f
d6229360-06fc-0000-805c-566ff2f30005    7       19      2       2       6       7       662     2025-01-20 07:21:09.020604      662     2025-01-20 07:21:09.020604      f
d6229360-06fc-0000-805c-566ff2f30006    7       19      3       3       8       9999    662     2025-01-20 07:21:09.020604      662     2025-01-20 07:21:09.020604      f
d6229360-06fc-0000-805c-566ff2f30007    8       \N      1       1       1       5       662     2025-01-20 07:21:09.020604      662     2025-01-20 07:21:09.020604      f
d6229360-06fc-0000-805c-566ff2f30008    8       \N      2       2       6       7       662     2025-01-20 07:21:09.020604      662     2025-01-20 07:21:09.020604      f
d6229360-06fc-0000-805c-566ff2f30009    8       \N      3       3       8       9999    662     2025-01-20 07:21:09.020604      662     2025-01-20 07:21:09.020604      f
\.

*/

/*Galko: Это версия с проверкой существования данных */


BEGIN TRANSACTION;

-- Индексируем уникальное поле UUID
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_regulatory_deadline_price_uuid') THEN
        CREATE UNIQUE INDEX idx_regulatory_deadline_price_uuid ON public.regulatory_deadline_price USING BTREE (uuid);
    END IF;
END $$;


-- Создаем временную таблицу для хранения входных данных
WITH input_data AS (
    VALUES
    ('d6229360-06fc-0000-805c-566ff2f30001'::UUID, 10, 19, 1, 1, 1, 5, 662, '2025-01-20 07:21:09.020604'::timestamp, 662, '2025-01-20 07:21:09.020604'::timestamp, FALSE),
    ('d6229360-06fc-0000-805c-566ff2f30002'::UUID, 10, 19, 2, 2, 6, 7, 662, '2025-01-20 07:21:09.020604'::timestamp, 662, '2025-01-20 07:21:09.020604'::timestamp, FALSE),
    ('d6229360-06fc-0000-805c-566ff2f30003'::UUID, 10, 19, 3, 3, 8, 9999, 662, '2025-01-20 07:21:09.020604'::timestamp, 662, '2025-01-20 07:21:09.020604'::timestamp, FALSE),
    ('d6229360-06fc-0000-805c-566ff2f30004'::UUID, 7, 19, 1, 1, 1, 5, 662, '2025-01-20 07:21:09.020604'::timestamp, 662, '2025-01-20 07:21:09.020604'::timestamp, FALSE),
    ('d6229360-06fc-0000-805c-566ff2f30005'::UUID, 7, 19, 2, 2, 6, 7, 662, '2025-01-20 07:21:09.020604'::timestamp, 662, '2025-01-20 07:21:09.020604'::timestamp, FALSE),
    ('d6229360-06fc-0000-805c-566ff2f30006'::UUID, 7, 19, 3, 3, 8, 9999, 662, '2025-01-20 07:21:09.020604'::timestamp, 662, '2025-01-20 07:21:09.020604'::timestamp, FALSE),
    ('d6229360-06fc-0000-805c-566ff2f30007'::UUID, 8, NULL, 1, 1, 1, 5, 662, '2025-01-20 07:21:09.020604'::timestamp, 662, '2025-01-20 07:21:09.020604'::timestamp, FALSE),
    ('d6229360-06fc-0000-805c-566ff2f30008'::UUID, 8, NULL, 2, 2, 6, 7, 662, '2025-01-20 07:21:09.020604'::timestamp, 662, '2025-01-20 07:21:09.020604'::timestamp, FALSE),
    ('d6229360-06fc-0000-805c-566ff2f30009'::UUID, 8, NULL, 3, 3, 8, 9999, 662, '2025-01-20 07:21:09.020604'::timestamp, 662, '2025-01-20 07:21:09.020604'::timestamp, FALSE)
)

-- Выполняем вставку данных с условием уникальности по полю uuid
INSERT INTO public.regulatory_deadline_price(uuid, section, field_id, color_scheme_id, type_criticality, start_day, end_day, created_by, created_at, changed_by, changed_at, status)
SELECT * FROM input_data
ON CONFLICT (uuid) DO NOTHING;


COMMIT;