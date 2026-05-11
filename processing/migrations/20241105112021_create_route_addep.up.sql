

CREATE TABLE public.route_addep (
    uuid uuid NOT NULL,
    route_id integer NOT NULL,
    department_id integer NOT NULL,
    division_id integer NOT NULL,
    is_removed boolean DEFAULT false NOT NULL,
    created_at timestamp without time zone DEFAULT '1900-01-01 00:00:00'::timestamp without time zone NOT NULL,
    changed_at timestamp without time zone DEFAULT '1900-01-01 00:00:00'::timestamp without time zone NOT NULL,
    created_by integer DEFAULT 0 NOT NULL,
    changed_by integer DEFAULT 0 NOT NULL
);

ALTER TABLE ONLY public.route_addep
    ADD CONSTRAINT route_addep_pkey PRIMARY KEY (uuid);

COMMENT ON TABLE public.route_addep IS 'Дополнительные ПД и Подразделения ПД';



COMMENT ON COLUMN public.route_addep.uuid IS 'Уникальный идентификатор записи';



COMMENT ON COLUMN public.route_addep.route_id IS 'Номер маршрута';



COMMENT ON COLUMN public.route_addep.department_id IS 'Код согласующей орг. единицы (уровень Департамента) дополнительный';



COMMENT ON COLUMN public.route_addep.division_id IS 'Код согласующего подразделения орг. Единицы дополнительный';



COMMENT ON COLUMN public.route_addep.is_removed IS 'Признак удаления записи';



COMMENT ON COLUMN public.route_addep.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';



COMMENT ON COLUMN public.route_addep.changed_at IS 'Код пользователя ответственного исполнителя, создавшего объект';



COMMENT ON COLUMN public.route_addep.created_by IS 'Дата и время изменения. Автоматически присваивается при изменении объекта';



COMMENT ON COLUMN public.route_addep.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
