-- Adds entry validity date range and is_removed property to OKPD2

ALTER TABLE public.okpd2
    ADD COLUMN is_removed BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN from_date DATE DEFAULT '1900-01-01'::DATE NOT NULL,
    ADD COLUMN to_date DATE DEFAULT '1900-01-01'::DATE NOT NULL;

UPDATE public.okpd2 SET to_date = '9999-12-31'::DATE;

-- Adds entry validity date range to Category

ALTER TABLE public.category
    ADD COLUMN from_date DATE DEFAULT '1900-01-01'::DATE NOT NULL,
    ADD COLUMN to_date DATE DEFAULT '1900-01-01'::DATE NOT NULL;

UPDATE public.category SET to_date = '9999-12-31'::DATE;
