-- Removes entry validity date range and is_removed property from OKPD2

ALTER TABLE public.okpd2
    DROP COLUMN is_removed,
    DROP COLUMN from_date,
    DROP COLUMN to_date;

-- Removes entry validity date range from Category

ALTER TABLE public.category
    DROP COLUMN from_date,
    DROP COLUMN to_date;
