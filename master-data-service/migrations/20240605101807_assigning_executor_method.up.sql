-- Table: public.assigning_executor
CREATE TABLE public.assigning_executor_method
(
	id SMALLSERIAL PRIMARY KEY,
	uuid uuid NOT NULL,
	name VARCHAR(250) NOT NULL DEFAULT '',
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.assigning_executor_method IS 'Справочник "Способ назначения исполнителя"';

COMMENT ON COLUMN public.assigning_executor_method.uuid IS 'UID Записи';
COMMENT ON COLUMN public.assigning_executor_method.id IS 'ID Записи';
COMMENT ON COLUMN public.assigning_executor_method.name IS 'Наименование';
COMMENT ON COLUMN public.assigning_executor_method.is_removed IS 'Запись удалена';

COMMENT ON COLUMN public.assigning_executor_method.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.assigning_executor_method.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.assigning_executor_method.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.assigning_executor_method.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

INSERT INTO public.assigning_executor_method(uuid, name, changed_at, created_at, changed_by, created_by)
	VALUES
	('d7cf2532-2c90-4296-9b5b-7afab7d05cf6', 'Выполнено автоматическое назначение исполнителя', now(), now(), 1, 1),
	('1af63b5a-4327-4222-83ce-0034e608575d', 'Выполнено ручное назначение исполнителя', now(), now(), 1, 1),
	('8e20b856-3395-48f9-bb36-224a1a485cfd', 'Автоматически назначенный исполнитель изменен вручную', now(), now(), 1, 1)
;
