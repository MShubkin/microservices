-- Table: public.status_sample_conclusion
-- See https://rcportal.inlinegroup.ru/web#id=2465&cids=1&model=project.task&view_type=form notes for details
CREATE TABLE public.status_sample_conclusion (
    id SMALLSERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL DEFAULT '',
    color_scheme_id SMALLINT NOT NULL DEFAULT 0, -- TODO: что это ?
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.status_sample_conclusion IS 'Справочник "Статусы шаблона"';

COMMENT ON COLUMN public.status_sample_conclusion.id IS 'Идентификатор Значения';
COMMENT ON COLUMN public.status_sample_conclusion.name IS 'Наименование типа объекта';
COMMENT ON COLUMN public.status_sample_conclusion.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.status_sample_conclusion.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.status_sample_conclusion.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.status_sample_conclusion.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.status_sample_conclusion.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

INSERT INTO
    public.status_sample_conclusion(name, changed_at, created_at, changed_by, created_by)
VALUES
    ('Создан', now(), now(), 1, 1),
    ('Изменен', now(), now(), 1, 1),
    ('Согласован', now(), now(), 1, 1),
    ('Отклонен', now(), now(), 1, 1)
;
