

CREATE TABLE public.field_history(
    id BIGSERIAL PRIMARY KEY NOT NULL UNIQUE,
    record_uuid uuid NOT NULL,
    table_name TEXT NOT NULL,
    field_name TEXT NOT NULL,
    field_value jsonb,
    record_status SMALLINT NOT NULL,
    created_at timestamp NOT NULL,
    created_by INTEGER NOT NULL
)
TABLESPACE pg_default;

COMMENT ON TABLE public.field_history
    is 'Таблица истории изменений полей записей.';

COMMENT ON COLUMN public.field_history.id
    is 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.field_history.record_uuid
    is 'Уникальный идентификатор записи в которой поле изменено';
COMMENT ON COLUMN public.field_history.table_name
    is 'Именование таблицы изменённой записи';
COMMENT ON COLUMN public.field_history.field_name
    is 'Именование поля которое изменёно в записи';
COMMENT ON COLUMN public.field_history.field_value
    is 'Значение поля которое изменёно в записи';
COMMENT ON COLUMN public.field_history.record_status
    is 'Статус изменение';
COMMENT ON COLUMN public.field_history.created_at
    is 'Timestamp создание записи `field_history`';
COMMENT ON COLUMN public.field_history.created_by
    is 'Идентификатор того кто изменил';
