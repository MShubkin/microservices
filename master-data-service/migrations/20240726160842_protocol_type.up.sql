-- Table: public.protocol_type
CREATE TABLE public.protocol_type (
  id SMALLSERIAL PRIMARY KEY,
  uuid uuid NOT NULL,
  name VARCHAR(250) DEFAULT '',
  is_removed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.protocol_type IS 'Справочник «Тип Протокола»';

COMMENT ON COLUMN public.protocol_type.uuid IS 'Уникальный идентификатор записи';

COMMENT ON COLUMN public.protocol_type.id IS 'Значения: «Тип Протокола»:
1 - Протокол очного заседания СК
2 - Протокол заочного заседания СК';

COMMENT ON COLUMN public.protocol_type.name IS 'Значения: «Тип Протокола»:
1 - Протокол очного заседания СК
2 - Протокол заочного заседания СК';

COMMENT ON COLUMN public.protocol_type.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.protocol_type.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.protocol_type.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.protocol_type.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';
COMMENT ON COLUMN public.protocol_type.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';

INSERT INTO
  public.protocol_type(uuid, id, name, created_at, changed_at, created_by, changed_by)
VALUES
  ('736cd972-7e03-4eb6-b789-fbc5bb3fa116', 1, 'Протокол очного заседания СК', now(), now(), 1, 1),
  ('45899a1c-14da-4105-93c5-ee19e8338dc1', 2, 'Протокол заочного заседания СК', now(), now(), 1, 1)
;
