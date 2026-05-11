-- Table: public.critical_type_color_scheme
CREATE TABLE public.critical_type_color_scheme (
    id SMALLSERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL DEFAULT '',
    type_id SMALLINT NOT NULL,
    color_code VARCHAR(6) NOT NULL DEFAULT '',
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.critical_type_color_scheme IS 'Справочник "Цветовые схемы критичности"';
COMMENT ON COLUMN public.critical_type_color_scheme.id IS 'Идентификатор Значения';
COMMENT ON COLUMN public.critical_type_color_scheme.name IS 'Наименование объекта';
COMMENT ON COLUMN public.critical_type_color_scheme.type_id IS 'Идентификатор типа критичности';
COMMENT ON COLUMN public.critical_type_color_scheme.color_code IS 'Код цветовой схемы';
COMMENT ON COLUMN public.critical_type_color_scheme.is_removed IS 'Признак удаления записи';
COMMENT ON COLUMN public.critical_type_color_scheme.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.critical_type_color_scheme.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.critical_type_color_scheme.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.critical_type_color_scheme.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

INSERT INTO
    public.critical_type_color_scheme(name, type_id, color_code, changed_at, created_at, changed_by, created_by)
VALUES
    ('Нормативные сроки анализа цены не нарушены', 1, '009245', now(), now(), 1, 1),
    ('Нормативные сроки анализа цены истекают', 2, 'F59E2A', now(), now(), 1, 1),
    ('Нормативные сроки анализа цены нарушены', 3, 'C1272D', now(), now(), 1, 1)
;
