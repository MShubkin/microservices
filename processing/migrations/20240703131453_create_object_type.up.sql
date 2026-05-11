

CREATE TABLE public.object_type (
  id SMALLINT NOT NULL PRIMARY KEY,
  sort_code BIGINT NOT NULL,
  -- TODO : Maybe generate with number_range?
  "value" SMALLINT NOT NULL,
  created_at timestamp without time zone NOT NULL,
  changed_at timestamp without time zone NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.object_type IS 'Таблица справочник значений типов участника';

COMMENT ON COLUMN public.object_type.sort_code IS 'Порядковый номер типов участника';

COMMENT ON COLUMN public.object_type.id IS 'Уникальный идентификатор записи';

COMMENT ON COLUMN public.object_type."value" IS 'Описание статуса участника';

COMMENT ON COLUMN public.object_type.created_by IS 'Идентификатор создателя';

COMMENT ON COLUMN public.object_type.changed_by IS 'Идентификатор того, кто изменил';

COMMENT ON COLUMN public.object_type.created_at IS 'Дата создания';

COMMENT ON COLUMN public.object_type.changed_at IS 'Дата изменения';
