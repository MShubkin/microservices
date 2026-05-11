-- Table: public.analysis_method
CREATE TABLE public.analysis_method
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

COMMENT ON TABLE public.analysis_method IS 'Справочник "Способ анализа"';

COMMENT ON COLUMN public.analysis_method.uuid IS 'UID Записи';
COMMENT ON COLUMN public.analysis_method.id IS 'ID Записи';
COMMENT ON COLUMN public.analysis_method.name IS 'Наименование';
COMMENT ON COLUMN public.analysis_method.is_removed IS 'Запись удалена';

COMMENT ON COLUMN public.analysis_method.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.analysis_method.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.analysis_method.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.analysis_method.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

INSERT INTO public.analysis_method(uuid, name, changed_at, created_at, changed_by, created_by)
	VALUES
	('f175d90a-4045-40d2-8540-35de03c084ed', 'Без запроса цен', now(), now(), 1, 1),
	('09771790-24a6-4bc7-9607-9c6f08d827e4', 'С запросом цен', now(), now(), 1, 1)
;
