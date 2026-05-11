

CREATE TABLE public.favourite_plans_by_id(
  uuid uuid NOT NULL PRIMARY KEY UNIQUE,
  plan_uuid uuid NOT NULL,
  user_id INTEGER NOT NULL,
  "status" SMALLINT NOT NULL,
  created_at timestamp without time zone NOT NULL,
  changed_at timestamp without time zone NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.favourite_plans_by_id IS 'Таблица популярных закупок';

COMMENT ON COLUMN public.favourite_plans_by_id.uuid IS 'Уникальный идентификатор записи';

COMMENT ON COLUMN public.favourite_plans_by_id.plan_uuid IS 'Уникальный идентификатор привязанного ППЗ';

COMMENT ON COLUMN public.favourite_plans_by_id.user_id IS 'Уникальный идентификатор записи';

COMMENT ON COLUMN public.favourite_plans_by_id."status" IS 'Статуса объекта, ссылка на object_status.id';

COMMENT ON COLUMN public.favourite_plans_by_id.created_by IS 'Идентификатор создателя';

COMMENT ON COLUMN public.favourite_plans_by_id.changed_by IS 'Идентификатор того кто изменил';

COMMENT ON COLUMN public.favourite_plans_by_id.created_at IS 'Дата создания';

COMMENT ON COLUMN public.favourite_plans_by_id.changed_at IS 'Дата изменения';
