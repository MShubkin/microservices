CREATE TABLE public.route_list (
    type_id smallint NOT NULL,
    uuid uuid NOT NULL,
    id bigint NOT NULL,
    name_short character varying(500),
    is_exception boolean DEFAULT false NOT NULL,
    is_active boolean DEFAULT false NOT NULL,
    is_removed boolean DEFAULT false NOT NULL,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    created_by integer NOT NULL,
    changed_by integer NOT NULL,
    route_type_id smallint DEFAULT 0 NOT NULL
);

ALTER TABLE public.route_list ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.route_list_id_seq
    START WITH 9100000001
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);

ALTER TABLE ONLY public.route_list
    ADD CONSTRAINT route_list_pkey PRIMARY KEY (uuid);

COMMENT ON TABLE public.route_list IS 'Справочник "Перечень маршрутов согласования"';

COMMENT ON COLUMN public.route_list.type_id IS 'Значение кешируемого справочника route_type';
COMMENT ON COLUMN public.route_list.uuid IS 'Уникальный идентификатор';
COMMENT ON COLUMN public.route_list.id IS 'Номер маршрута';
COMMENT ON COLUMN public.route_list.name_short IS 'Краткое наименование маршрута';
COMMENT ON COLUMN public.route_list.is_exception IS 'Исключения';
COMMENT ON COLUMN public.route_list.is_active IS 'Индикатор Вкл/Искл';
COMMENT ON COLUMN public.route_list.is_removed IS 'Признак удаления записи';
COMMENT ON COLUMN public.route_list.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.route_list.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.route_list.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';
COMMENT ON COLUMN public.route_list.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';

