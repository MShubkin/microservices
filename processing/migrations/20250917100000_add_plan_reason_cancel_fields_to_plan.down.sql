ALTER TABLE public.plan 
DROP COLUMN plan_reason_cancel_id integer,
DROP COLUMN plan_replaced_id bigint;

ALTER TABLE public.plan_version 
DROP COLUMN plan_reason_cancel_id integer,
DROP COLUMN plan_replaced_id bigint;