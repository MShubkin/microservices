

CREATE TABLE public.executor_method
(
    id SMALLINT NOT NULL PRIMARY KEY,
    "value" VARCHAR(63) NOT NULL,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    -- TODO: We should probably be inserting some kind of id here
    --       Then we can return to varchar(10)
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
)
TABLESPACE pg_default;

COMMENT ON TABLE public.executor_method
    IS 'Таблица способов согласование';

COMMENT ON COLUMN public.executor_method.id
    IS 'Идентификатор записи';

COMMENT ON COLUMN public.executor_method."value"
    IS 'Значение (текст) записи';

COMMENT ON COLUMN public.executor_method.created_by
    IS 'Идентификатор создателя';

COMMENT ON COLUMN public.executor_method.changed_by
    IS 'Идентификатор того кто изменил';

COMMENT ON COLUMN public.executor_method.created_at
    IS 'Дата создания';

COMMENT ON COLUMN public.executor_method.changed_at
    IS 'Дата изменения';
