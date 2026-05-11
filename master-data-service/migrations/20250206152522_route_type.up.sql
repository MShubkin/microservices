CREATE TABLE public.route_type (
    id SMALLINT NOT NULL DEFAULT 0,
    text VARCHAR(120) NOT NULL DEFAULT '',
    changed_by INTEGER NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    is_removed BOOLEAN NOT NULL DEFAULT FALSE
);

INSERT INTO public.route_type (id, text, changed_by, changed_at) VALUES
(1, 'Наименование типа маршрута', 1, now()),
(2, 'Наименование типа маршрута', 1, now()),
(3, 'Наименование типа маршрута', 1, now());

COMMENT ON TABLE public.route_type IS 'Типы маршрутов согласвания';
COMMENT ON COLUMN public.route_type.id IS 'Тип маршрута';
COMMENT ON COLUMN public.route_type.text IS 'Наименование типа маршрута';
COMMENT ON COLUMN public.route_type.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.route_type.changed_at IS 'Дата и время изменения записи';
COMMENT ON COLUMN public.route_type.is_removed IS 'Признак удаленной записи таблицы';
