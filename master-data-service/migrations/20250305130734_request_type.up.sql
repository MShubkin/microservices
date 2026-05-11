-- Table: public.request_type
CREATE TABLE public.request_type
(
    id SMALLSERIAL PRIMARY KEY,
    name VARCHAR(250) NOT NULL DEFAULT '',
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
    );

COMMENT ON TABLE public.request_type IS 'Справочник "Тип ЗЦИ"';

COMMENT ON COLUMN public.request_type.id IS 'ID Записи';
COMMENT ON COLUMN public.request_type.name IS 'Наименование типа ЗЦИ';
COMMENT ON COLUMN public.request_type.is_removed IS 'Запись удалена';

COMMENT ON COLUMN public.request_type.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.request_type.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.request_type.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';
COMMENT ON COLUMN public.request_type.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';

INSERT INTO public.request_type(name, changed_at, created_at, changed_by, created_by)
VALUES
    ('Открытый', now(), now(), 1, 1),
    ('Закрытый', now(), now(), 1, 1),
    ('Открытый санкционный', now(), now(), 1, 1),
    ('Закрытый санкционный', now(), now(), 1, 1)
;
