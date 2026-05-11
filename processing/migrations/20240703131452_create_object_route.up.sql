

CREATE TABLE public.object_route
(
    uuid uuid NOT NULL PRIMARY KEY,
    route_uuid uuid NOT NULL,
    -- Should be 'ПД' or 'АЦ'.
    designation_type char(2) NOT NULL,
    responsible_unit_id BIGINT NOT NULL,
    -- Somehow joins `department.price_department_id`.
    price_department_id BIGINT,
    executor_id BIGINT NOT NULL,
    -- Somehow joins `executor_method.id`
    executor_method_id SMALLINT NOT NULL,
    "status" SMALLINT NOT NULL, 
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    -- TODO: We should probably be inserting some kind of id here
    --       Then we can return to varchar(10)
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
)
TABLESPACE pg_default;

COMMENT ON TABLE public.object_route
    IS 'Таблица путей согласование';

COMMENT ON COLUMN public.object_route.uuid
    IS 'Уникальный идентификатор записи';

COMMENT ON COLUMN public.object_route.designation_type
    IS 'Тип назначения: ПД или АЦ';

COMMENT ON COLUMN public.object_route.responsible_unit_id
    IS 'Подразделение, ответственное за анализ цены.';

COMMENT ON COLUMN public.object_route.price_department_id
    IS 'Отдел: Соединён с `department.pricing_organization_unit_id`';

COMMENT ON COLUMN public.object_route.executor_method_id
    IS 'Способ назначения исполнителя: Соединён с `executor_method.uuid';

COMMENT ON COLUMN public.object_route.created_by
    IS 'Идентификатор создателя';

COMMENT ON COLUMN public.object_route.changed_by
    IS 'Идентификатор того кто изменил';

COMMENT ON COLUMN public.object_route.created_at
    IS 'Дата создания';

COMMENT ON COLUMN public.object_route.changed_at
    IS 'Дата изменения';
