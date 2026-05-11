-- Table: public.ppz_type
CREATE TABLE public.ppz_type
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

COMMENT ON TABLE public.ppz_type IS 'Справочник "Тип ППЗ"';

COMMENT ON COLUMN public.ppz_type.uuid IS 'UID Записи';
COMMENT ON COLUMN public.ppz_type.id IS 'ID Записи';
COMMENT ON COLUMN public.ppz_type.name IS 'Наименование';
COMMENT ON COLUMN public.ppz_type.is_removed IS 'Запись удалена';

COMMENT ON COLUMN public.ppz_type.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.ppz_type.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.ppz_type.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.ppz_type.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

INSERT INTO public.ppz_type(uuid, name, changed_at, created_at, changed_by, created_by)
	VALUES
	('02987045-a9b8-4528-8595-62fb17c5d252', 'МТР', now(), now(), 1, 1),
	('98d98d65-9153-4f83-936a-172f93667825', 'Работа', now(), now(), 1, 1),
	('9f72f82b-9679-452f-a617-9b9e8c7630ad', 'Услуга', now(), now(), 1, 1)
;
