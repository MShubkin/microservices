ALTER TABLE public.plan 
RENAME COLUMN plan_reason_cancel_id TO reason_cancel_id;

ALTER TABLE public.plan 
RENAME COLUMN plan_replaced_id TO replaced_id;

ALTER TABLE public.plan_version 
RENAME COLUMN plan_reason_cancel_id TO reason_cancel_id;

ALTER TABLE public.plan_version 
RENAME COLUMN plan_replaced_id TO replaced_id;