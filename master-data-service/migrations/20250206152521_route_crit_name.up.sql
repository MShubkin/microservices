CREATE TABLE public.route_crit_name (
    id SMALLINT NOT NULL,
    text VARCHAR(120) NOT NULL DEFAULT '',
    changed_by INTEGER NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    is_removed BOOLEAN NOT NULL DEFAULT FALSE
);

INSERT INTO public.route_crit_name (id, text, changed_by, changed_at) VALUES
(1, 'Наименование критерия', 1, now()),
(2, 'Наименование критерия', 1, now()),
(3, 'Наименование критерия', 1, now());

COMMENT ON TABLE public.route_crit_name IS 'Критерии маршрутов согласования';
COMMENT ON COLUMN public.route_crit_name.id IS 'Идентификатор критерия';
COMMENT ON COLUMN public.route_crit_name.text IS 'Наименование критерия';
COMMENT ON COLUMN public.route_crit_name.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.route_crit_name.changed_at IS 'Дата и время изменения записи';
COMMENT ON COLUMN public.route_crit_name.is_removed IS 'Признак удаленной записи таблицы';
