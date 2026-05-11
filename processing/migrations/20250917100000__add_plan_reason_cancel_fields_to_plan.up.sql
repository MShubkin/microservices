ALTER TABLE public.plan 
ADD COLUMN plan_reason_cancel_id integer,
ADD COLUMN plan_replaced_id bigint;

ALTER TABLE public.plan_version 
ADD COLUMN plan_reason_cancel_id integer,
ADD COLUMN plan_replaced_id bigint;