-- Table: public.status_history


CREATE TABLE public.status_history (
  uuid uuid NOT NULL PRIMARY KEY,
  object_uuid uuid NOT NULL,
  status_id SMALLINT NOT NULL DEFAULT 0,
  comment VARCHAR(255) DEFAULT '',
  created_at timestamp without time zone NOT NULL,
  created_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.status_history IS 'История изменения статуса';

COMMENT ON COLUMN public.status_history.uuid IS 'Уникальный идентификатор записи истории';

COMMENT ON COLUMN public.status_history.object_uuid IS 'Уникальный идентификатор объекта (Повестка / Протокол / ППЗ / ДС)';

COMMENT ON COLUMN public.status_history.status_id IS 'Новый статус';

COMMENT ON COLUMN public.status_history.comment IS 'Комментарий';

COMMENT ON COLUMN public.status_history.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

COMMENT ON COLUMN public.status_history.created_at IS 'Дата и время создания.
Автоматически присваивается при создании объекта';
