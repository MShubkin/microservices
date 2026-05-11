-- Table: public.expert_conclusion_type
CREATE TABLE public.expert_conclusion_type (
    id SMALLSERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL DEFAULT '',
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.expert_conclusion_type IS 'Справочник "Типы заключений эксперта"';

COMMENT ON COLUMN public.expert_conclusion_type.id IS 'Идентификатор Значения';
COMMENT ON COLUMN public.expert_conclusion_type.name IS 'Наименование типа объекта';
COMMENT ON COLUMN public.expert_conclusion_type.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.expert_conclusion_type.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.expert_conclusion_type.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.expert_conclusion_type.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.expert_conclusion_type.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

INSERT INTO
    public.expert_conclusion_type(name, changed_at, created_at, changed_by, created_by)
VALUES
    ('Согласовано с заявленной стоимостью', now(), now(), 1, 1),
    ('Согласовано со снижением стоимости', now(), now(), 1, 1),
    ('Согласовано с повышением стоимости', now(), now(), 1, 1),
    ('Возврат заказчику', now(), now(), 1, 1),
    ('Запрос документации', now(), now(), 1, 1)
;
