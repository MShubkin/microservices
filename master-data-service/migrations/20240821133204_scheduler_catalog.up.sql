-- Table: public.scheduler_catalog
CREATE TABLE public.scheduler_catalog
(
    id SERIAL PRIMARY KEY,
    event_name VARCHAR(250) NOT NULL DEFAULT '',
    event_date DATE NOT NULL,
    period_time SMALLINT NOT NULL,
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.scheduler_catalog IS 'Справочник "Производственный календарь"';

COMMENT ON COLUMN public.scheduler_catalog.id IS 'ID Записи';
COMMENT ON COLUMN public.scheduler_catalog.event_name IS 'Наименование события';
COMMENT ON COLUMN public.scheduler_catalog.event_date IS 'Дата события';
COMMENT ON COLUMN public.scheduler_catalog.period_time IS 'Период времени к которому относится позиция (Год)';
COMMENT ON COLUMN public.scheduler_catalog.is_removed IS 'Запись удалена';

COMMENT ON COLUMN public.scheduler_catalog.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.scheduler_catalog.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.scheduler_catalog.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';
COMMENT ON COLUMN public.scheduler_catalog.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';

INSERT INTO public.scheduler_catalog(event_name, event_date, period_time, changed_at, created_at, changed_by, created_by)
	VALUES
	('Peace. Labour. May', '2024-05-01', 2024, now(), now(), 1, 1)
;
