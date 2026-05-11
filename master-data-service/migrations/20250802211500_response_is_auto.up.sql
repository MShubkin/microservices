ALTER TABLE public.response
    ADD COLUMN is_auto BOOLEAN NOT NULL DEFAULT false;

UPDATE public.response 
    SET is_auto = true 
    WHERE id = 5;
