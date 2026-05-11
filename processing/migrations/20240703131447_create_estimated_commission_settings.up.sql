

-- There is literally no data on this table. All the fields
-- except `commission_kind_id` have types made up on the spot.
CREATE TABLE public.estimated_commission_settings(
    -- joins `plan.commission_kind_id`
    commission_kind_id SMALLINT NOT NULL PRIMARY KEY,
    parameter INTEGER NOT NULL,
    selection_option VARCHAR(63),
    content_field_high VARCHAR(63),
    content_field_low VARCHAR(63)
)
TABLESPACE pg_default;

COMMENT ON TABLE public.estimated_commission_settings
    IS 'Таблица настроек СК';

COMMENT ON COLUMN public.estimated_commission_settings.commission_kind_id
    IS 'Индификатор. Соединение с plan.commission_kind_id'
