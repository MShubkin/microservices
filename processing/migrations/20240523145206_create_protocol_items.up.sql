

CREATE TABLE public.protocol_item (
  uuid uuid NOT NULL PRIMARY KEY,
  protocol_uuid uuid NOT NULL,
  source_uuid uuid NOT NULL,
  number BIGINT NOT NULL,
  is_registered_by_d647 BOOLEAN NOT NULL DEFAULT false,
  is_removed BOOLEAN NOT NULL DEFAULT false,
  is_excluded BOOLEAN NOT NULL DEFAULT false,
  result_id SMALLINT DEFAULT 0,
  sum_excluded_vat BIGINT,
  pricing_sum_excluded_vat BIGINT,
  commission_sum_excluded_vat BIGINT,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.protocol_item IS 'Таблица предметов протокола';

COMMENT ON COLUMN public.protocol_item.uuid IS 'Уникальный идентификатор записи';

COMMENT ON COLUMN public.protocol_item.source_uuid IS 'Уникальный идентификатор ППЗ/ДС';

COMMENT ON COLUMN public.protocol_item.protocol_uuid IS 'Уникальный идентификатор Протокола';

COMMENT ON COLUMN public.protocol_item.number IS 'Внешний порядковый номер';

COMMENT ON COLUMN public.protocol_item.is_registered_by_d647 IS 'Признак нахождения ППЗ/ДС в Реестре Д647';

COMMENT ON COLUMN public.protocol_item.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.protocol_item.is_excluded IS 'Признак "Снято с рассмотрения".
Заполняется пользователем вручную или автоматически на экране ';

COMMENT ON COLUMN public.protocol_item.result_id IS 'Справочник «Решение комиссии»:
-  Утверждено
-  Согласовано с корректировкой стоимости
-  Не согласовано. Вернуть Эксперту
-  Аннулировать';

COMMENT ON COLUMN public.protocol_item.commission_sum_excluded_vat IS 'Сумма СК, без НДС';

COMMENT ON COLUMN public.protocol_item.created_by IS 'Логин пользователя ответственного исполнителя, создавшего запись';

COMMENT ON COLUMN public.protocol_item.changed_by IS 'Логин пользователя ответственного исполнителя, изменившего запись';

COMMENT ON COLUMN public.protocol_item.created_at IS 'Дата и время создания.
Автоматически присваивается при создании записи';

COMMENT ON COLUMN public.protocol_item.changed_at IS 'Дата и время изменения.
Автоматически присваивается при создании записи';
