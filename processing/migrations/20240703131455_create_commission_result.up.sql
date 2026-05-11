-- Table: public.estimated_commission_result


CREATE TABLE public.estimated_commission_result (
	uuid uuid NOT NULL PRIMARY KEY,
	result_id SMALLINT DEFAULT 0,
	name VARCHAR(250) DEFAULT '',
	is_removed boolean NOT NULL DEFAULT false,
	changed_at timestamp without time zone NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.estimated_commission_result IS 'Справочник «Решения Сметной комиссии по ППЗ/ДС»"';

COMMENT ON COLUMN public.estimated_commission_result.uuid IS 'UID Записи';

COMMENT ON COLUMN public.estimated_commission_result.name IS 'Наименование';

COMMENT ON COLUMN public.estimated_commission_result.changed_at IS 'Дата изменения';

COMMENT ON COLUMN public.estimated_commission_result.is_removed IS 'Запись удалена';
