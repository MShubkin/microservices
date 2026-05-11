-- Table: public.price_information_request_type
CREATE TABLE public.price_information_request_type
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

COMMENT ON TABLE public.price_information_request_type IS 'Справочник "Тип ЗЦИ"';

COMMENT ON COLUMN public.price_information_request_type.uuid IS 'UID Записи';
COMMENT ON COLUMN public.price_information_request_type.id IS 'ID Записи';
COMMENT ON COLUMN public.price_information_request_type.name IS 'Наименование типа ЗЦИ';
COMMENT ON COLUMN public.price_information_request_type.is_removed IS 'Запись удалена';

COMMENT ON COLUMN public.price_information_request_type.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.price_information_request_type.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.price_information_request_type.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';
COMMENT ON COLUMN public.price_information_request_type.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';

INSERT INTO public.price_information_request_type(uuid, name, changed_at, created_at, changed_by, created_by)
	VALUES
	('0331aa6c-6a13-469d-b13a-c34824d07e72', 'Открытый', now(), now(), 1, 1),
	('df29d7db-507f-47e8-971f-2736178ce690', 'Закрытый', now(), now(), 1, 1),
	('6fedc424-4c49-49b7-8496-347b8d2bbaaf', 'Открытый санкционный', now(), now(), 1, 1),
	('6ea55999-2850-49ef-a58a-1692ff843b45', 'Закрытый санкционный', now(), now(), 1, 1)
;
