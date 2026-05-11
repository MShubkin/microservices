ALTER TABLE public.plan 
RENAME COLUMN replaced_id TO plan_reason_cancel_id;

ALTER TABLE public.plan 
RENAME COLUMN plan_replaced_id TO plan_replaced_id;

ALTER TABLE public.plan_version 
RENAME COLUMN reason_cancel_id TO plan_reason_cancel_id;

ALTER TABLE public.plan_version 
RENAME COLUMN replaced_id TO plan_replaced_id;