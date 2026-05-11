ALTER TABLE public.plan_reason_cancel 
ADD COLUMN check_reason_id_new smallint DEFAULT 0 NOT NULL;

UPDATE public.plan_reason_cancel 
SET check_reason_id_new = COALESCE(check_reason_id[1], 0);

ALTER TABLE public.plan_reason_cancel
DROP COLUMN check_reason_id;

ALTER TABLE public.plan_reason_cancel
RENAME COLUMN check_reason_id_new TO check_reason_id;