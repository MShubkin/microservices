-- Увеличить длинну code для новых кодов ВПЗ

ALTER TABLE public.category
    ALTER COLUMN code TYPE VARCHAR(10);
