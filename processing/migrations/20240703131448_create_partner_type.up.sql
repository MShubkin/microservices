

CREATE TABLE public.partner_type (
  user_id INTEGER NOT NULL PRIMARY KEY,
  -- TODO: Determine whether this is optimal.
  id BIGINT NOT NULL,
  -- TODO : Maybe generate with number_range?
  type_id SMALLINT NOT NULL,
  created_at timestamp without time zone NOT NULL,
  changed_at timestamp without time zone NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.partner_type IS 'Таблица типов участника';

COMMENT ON COLUMN public.partner_type.id IS 'Порядковый номер записи';

COMMENT ON COLUMN public.partner_type.user_id IS 'Код входа участника';

COMMENT ON COLUMN public.partner_type.type_id IS 'Код типа участника';

COMMENT ON COLUMN public.partner_type.created_by IS 'Идентификатор создателя';

COMMENT ON COLUMN public.partner_type.changed_by IS 'Идентификатор того кто изменил';

COMMENT ON COLUMN public.partner_type.created_at IS 'Дата создания';

COMMENT ON COLUMN public.partner_type.changed_at IS 'Дата изменения';
