

CREATE TABLE public.agenda_item (
  uuid uuid NOT NULL PRIMARY KEY,
  agenda_uuid uuid NOT NULL,
  source_uuid uuid NOT NULL,
  number BIGINT NOT NULL,
  is_registered_by_d647 BOOLEAN NOT NULL DEFAULT false,
  is_excluded BOOLEAN NOT NULL DEFAULT false,
  is_removed BOOLEAN NOT NULL DEFAULT false,
  reviewed_at timestamp without time zone,
  sum_excluded_vat BIGINT,
  pricing_sum_excluded_vat BIGINT,
  created_at timestamp without time zone NOT NULL,
  changed_at timestamp without time zone NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.agenda_item IS 'Таблица предметов повестки';

COMMENT ON COLUMN public.agenda_item.uuid IS 'Уникальный идентификатор записи';

COMMENT ON COLUMN public.agenda_item.agenda_uuid IS 'Уникальный идентификатор Повестки';

COMMENT ON COLUMN public.agenda_item.source_uuid IS 'Уникальный идентификатор ППЗ/ДС';

COMMENT ON COLUMN public.agenda_item.number IS 'Внешний порядковый номер';

COMMENT ON COLUMN public.agenda_item.is_registered_by_d647 IS 'Признак нахождения ППЗ/ДС в Реестре Д647';

COMMENT ON COLUMN public.agenda_item.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.agenda_item.is_excluded IS 'Признак "Снято с рассмотрения".
Заполняется пользователем вручную или автоматически на экране ';

COMMENT ON COLUMN public.agenda_item.reviewed_at IS 'Время проведения.
Заполняется пользователем вручную или с помощью кнопки «Задать время» на экране';

COMMENT ON COLUMN public.agenda_item.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

COMMENT ON COLUMN public.agenda_item.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';

COMMENT ON COLUMN public.agenda_item.created_at IS 'Дата и время создания.
Автоматически присваивается при создании объекта';

COMMENT ON COLUMN public.agenda_item.changed_at IS 'Дата и время изменения.
Автоматически присваивается при создании объекта';
