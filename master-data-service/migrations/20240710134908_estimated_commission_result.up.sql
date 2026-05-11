-- Table: public.estimated_commission_result
CREATE TABLE public.estimated_commission_result (
  id SMALLSERIAL PRIMARY KEY,
  uuid uuid NOT NULL,
  name VARCHAR(250) NOT NULL DEFAULT '',
  is_removed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.estimated_commission_result IS 'Справочник «Решения Сметной комиссии по ППЗ/ДС»"';

COMMENT ON COLUMN public.estimated_commission_result.uuid IS 'Уникальный идентификатор записи';

COMMENT ON COLUMN public.estimated_commission_result.name IS 'Значения:
1 - Утверждено
2 - Согласовано с корректировкой стоимости
3 - Не согласовано. Вернуть Эксперту АЦ
4 - Аннулировать';

COMMENT ON COLUMN public.estimated_commission_result.name IS 'Значения:
1 - Утверждено
2 - Согласовано с корректировкой стоимости
3 - Не согласовано. Вернуть Эксперту АЦ
4 - Аннулировать';

COMMENT ON COLUMN public.estimated_commission_result.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.estimated_commission_result.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.estimated_commission_result.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.estimated_commission_result.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';
COMMENT ON COLUMN public.estimated_commission_result.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';

INSERT INTO public.estimated_commission_result(uuid, name, created_at, changed_at, created_by, changed_by)
VALUES
  ('736cd972-7e03-4eb6-b789-fbc5bb3fa116', 'Утверждено', now(), now(), 1, 1),
  ('45899a1c-14da-4105-93c5-ee19e8338dc1', 'Согласовано с корректировкой стоимости', now(), now(), 1, 1),
  ('121181fa-d81b-4d03-a024-9456157cf4be', 'Не согласовано. Вернуть Эксперту АЦ', now(), now(), 1, 1),
  ('919022bf-149a-4454-b20a-9ceed25da6ee', 'Аннулировать', now(), now(), 1, 1)
;
