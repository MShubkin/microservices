BEGIN TRANSACTION;

-- Дропаем индекс
DROP INDEX IF EXISTS idx_regulatory_deadline_price_uuid;

-- Удаляем конкретные записи, добавленные в предыдущей миграции
DELETE FROM public.regulatory_deadline_price
WHERE uuid IN (
    'd6229360-06fc-0000-805c-566ff2f30001',
    'd6229360-06fc-0000-805c-566ff2f30002',
    'd6229360-06fc-0000-805c-566ff2f30003',
    'd6229360-06fc-0000-805c-566ff2f30004',
    'd6229360-06fc-0000-805c-566ff2f30005',
    'd6229360-06fc-0000-805c-566ff2f30006',
    'd6229360-06fc-0000-805c-566ff2f30007',
    'd6229360-06fc-0000-805c-566ff2f30008',
    'd6229360-06fc-0000-805c-566ff2f30009'
);

COMMIT;