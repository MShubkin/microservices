-- Table: public.estimated_commission_result
CREATE TABLE public.attachment_type (
  id SMALLSERIAL PRIMARY KEY,
  uuid uuid NOT NULL,
  name VARCHAR(250) NOT NULL DEFAULT '',
  is_removed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.attachment_type IS 'Справочник «Тип вложенного документа»';
COMMENT ON COLUMN public.attachment_type.uuid IS 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.attachment_type.id IS 'Значения:
1 - Повестка
2 - Протокол очного заседания СК
3 - Протокол заочного заседания СК
4 - Бюллетень';
COMMENT ON COLUMN public.attachment_type.name IS 'Значения:
1 - Повестка
2 - Протокол очного заседания СК
3 - Протокол заочного заседания СК
4 - Бюллетень';
COMMENT ON COLUMN public.attachment_type.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.attachment_type.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.attachment_type.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.attachment_type.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.attachment_type.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

INSERT INTO
    public.attachment_type(name, uuid, changed_at, created_at, changed_by, created_by)
VALUES
    ('Повестка', gen_random_uuid(), now(), now(), 1, 1),
    ('Протокол очного заседания СК', gen_random_uuid(), now(), now(), 1, 1),
    ('Протокол заочного заседания СК', gen_random_uuid(), now(), now(), 1, 1),
    ('Бюллетень', gen_random_uuid(), now(), now(), 1, 1)
;
