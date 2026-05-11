

CREATE TABLE public.agenda (
  uuid uuid NOT NULL PRIMARY KEY,
  id BIGINT NOT NULL,
  meeting_date date NOT NULL,
  status_id SMALLINT NOT NULL DEFAULT 0,
  pricing_organization_unit_id SMALLINT NOT NULL DEFAULT 0,
  is_removed BOOLEAN NOT NULL DEFAULT false,
  created_at timestamp without time zone NOT NULL,
  changed_at timestamp without time zone NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.agenda IS 'Таблица повесток АЦ';

COMMENT ON COLUMN public.agenda.uuid IS 'Уникальный идентификатор. 
Автоматически генерируется и присваивается при создании объекта';

COMMENT ON COLUMN public.agenda.id IS 'Системный номер. 
Автоматически присваивается при создании объекта';

COMMENT ON COLUMN public.agenda.meeting_date IS 'Дата Заседания Сметной комиссии';

COMMENT ON COLUMN public.agenda.status_id IS 'Статус объекта. Перечень значений приведен в таблице - Справочник «Статусы объекта»';

COMMENT ON COLUMN public.agenda.pricing_organization_unit_id IS 'Справочник «Департамент (организация) АЦ»:
-  Д646
-  Д647
-  ГПК
Автоматически определяется на основании Департамента (организации) проводившего АЦ';

COMMENT ON COLUMN public.agenda.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.agenda.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

COMMENT ON COLUMN public.agenda.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';

COMMENT ON COLUMN public.agenda.created_at IS 'Дата и время создания.
Автоматически присваивается при создании объекта';

COMMENT ON COLUMN public.agenda.changed_at IS 'Дата и время изменения.
Автоматически присваивается при создании объекта';
