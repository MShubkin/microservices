

CREATE TABLE public.partner_type_commission (
  uuid uuid NOT NULL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  role_id SMALLINT NOT NULL,
  protocol_type_id SMALLINT NOT NULL DEFAULT 0,
  created_at timestamp without time zone NOT NULL,
  changed_at timestamp without time zone NOT NULL,
  -- TODO: We should probably be inserting some kind of id here
  --       Then we can return to varchar(10)
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.partner_type_commission IS 'Настройка перечня участников Сметной комиссии';

COMMENT ON COLUMN public.partner_type_commission.uuid IS 'Уникальный идентификатор записи таблицы';

COMMENT ON COLUMN public.partner_type_commission.user_id IS 'Код члена СК';

COMMENT ON COLUMN public.partner_type_commission.role_id IS 'Код роли пользователя';

COMMENT ON COLUMN public.partner_type_commission.protocol_type_id IS 'Справочник «Тип Протокола»:
        1 -  Протокол очного заседания СК
        2 -  Протокол заочного заседания СК';

COMMENT ON COLUMN public.partner_type_commission.created_by IS 'Идентификатор создателя';

COMMENT ON COLUMN public.partner_type_commission.changed_by IS 'Идентификатор того кто изменил';

COMMENT ON COLUMN public.partner_type_commission.created_at IS 'Дата создания';

COMMENT ON COLUMN public.partner_type_commission.changed_at IS 'Дата изменения';
