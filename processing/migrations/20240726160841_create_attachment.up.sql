

CREATE TABLE public.attachment (
  uuid uuid NOT NULL PRIMARY KEY,
  object_uuid uuid NOT NULL,
  number SMALLINT NOT NULL,
  kind_id SMALLINT NOT NULL DEFAULT 0,
  name VARCHAR(255) NOT NULL DEFAULT '',
  parent_number SMALLINT,
  category_id SMALLINT NOT NULL DEFAULT 0,
  mime_id SMALLINT NOT NULL DEFAULT 0,
  size BIGINT NOT NULL,
  is_removed BOOLEAN NOT NULL DEFAULT false,
  is_classified BOOLEAN NOT NULL DEFAULT false,
  pricing_version SMALLINT NOT NULL DEFAULT 0,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL,
  created_at timestamp without time zone NOT NULL,
  changed_at timestamp without time zone NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.attachment IS 'Документы объектов';

COMMENT ON COLUMN public.attachment.uuid IS 'Уникальный идентификатор приложенного документа';

COMMENT ON COLUMN public.attachment.object_uuid IS 'Уникальный идентификатор Повестки / Протокола / ППЗ/ДС';

COMMENT ON COLUMN public.attachment.parent_number IS 'Связь с родителем(папкой)';

COMMENT ON COLUMN public.attachment.number IS 'Порядковая последовательность';

COMMENT ON COLUMN public.attachment.kind_id IS 'Файл/Папка/Ссылка';

COMMENT ON COLUMN public.attachment.name IS 'Наименование приложенного документа ';

COMMENT ON COLUMN public.attachment.category_id IS 'Справочник «Тип приложенного документа»:
-  Повестка
-  Протокол очного заседания СК
-  Протокол заочного заседания СК
-  Бюллетень';

COMMENT ON COLUMN public.attachment.mime_id IS 'Расширение файла';

COMMENT ON COLUMN public.attachment.size IS 'Размер файла (в байтах)';

COMMENT ON COLUMN public.attachment.is_removed IS 'Признак удаления приложенного документа';

COMMENT ON COLUMN public.attachment.is_classified IS 'Признак ИОД';

COMMENT ON COLUMN public.attachment.created_at IS 'Дата и время создания.
Автоматически присваивается при создании объекта';

COMMENT ON COLUMN public.attachment.changed_at IS 'Дата и время изменения.
Автоматически присваивается при создании объекта';

COMMENT ON COLUMN public.attachment.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

COMMENT ON COLUMN public.attachment.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
