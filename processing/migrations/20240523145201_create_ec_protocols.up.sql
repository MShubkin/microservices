

CREATE TABLE public.protocol (
  uuid uuid NOT NULL PRIMARY KEY,
  id BIGINT NOT NULL,
  protocol_type_id SMALLINT NOT NULL DEFAULT 0,
  registration_number VARCHAR(64),
  status_id SMALLINT NOT NULL,
  pricing_organization_unit_id SMALLINT NOT NULL DEFAULT 0,
  is_secret BOOLEAN NOT NULL DEFAULT false,
  is_removed BOOLEAN NOT NULL DEFAULT false,
  protocol_date DATE NOT NULL,
  created_at timestamp without time zone NOT NULL,
  changed_at timestamp without time zone NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.protocol IS 'Заголовок протокола';

COMMENT ON COLUMN public.protocol.uuid IS 'Уникальный идентификатор. 
Автоматически генерируется и присваивается при создании объекта';

COMMENT ON COLUMN public.protocol.id IS 'Системный номер. 
Автоматически присваивается при создании объекта';

COMMENT ON COLUMN public.protocol.protocol_type_id IS 'Справочник «Тип Протокола»:
1 -  Протокол очного заседания СК
2 -  Протокол заочного заседания СК';

COMMENT ON COLUMN public.protocol.registration_number IS 'Бумажный номер Протокола.
Заполняется пользователем вручную на экране';

COMMENT ON COLUMN public.protocol.status_id IS 'Статус объекта. Перечень значений приведен в таблице - Справочник «Статусы Протокола»';

COMMENT ON COLUMN public.protocol.pricing_organization_unit_id IS 'Справочник «Департамент (организация) АЦ»:
-  Д646
-  Д647
-  ГПК
Автоматически определяется на основании Департамента (организации) проводившего АЦ';

COMMENT ON COLUMN public.protocol.is_secret IS 'Признак Коммерческая тайна. Заполняется пользователем вручную на экране';

COMMENT ON COLUMN public.protocol.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.protocol.protocol_date IS 'Дата Заседания/Дата Протокола';

COMMENT ON COLUMN public.protocol.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

COMMENT ON COLUMN public.protocol.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';

COMMENT ON COLUMN public.protocol.created_at IS 'Дата и время создания.
Автоматически присваивается при создании объекта';

COMMENT ON COLUMN public.protocol.changed_at IS 'Дата и время изменения.
Автоматически присваивается при создании объекта';
