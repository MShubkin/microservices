

CREATE TABLE public.plan_retrospective (
    id bigint NOT NULL,
    plan_uuid uuid NOT NULL,
    plan_id bigint NOT NULL,
    plan_year smallint NOT NULL,
    plan_status smallint NOT NULL,
    id_ly bigint NOT NULL,
    uuid_ly uuid,
    is_removed boolean
);

ALTER TABLE ONLY public.plan_retrospective
    ADD CONSTRAINT plan_retrospective_pkey PRIMARY KEY (id_ly, plan_id);

ALTER TABLE public.plan_retrospective ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.plan_retrospective_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);

CREATE INDEX plan_retrospective_id_index ON public.plan_retrospective USING btree (id);

CREATE INDEX plan_retrospective_uuid_index ON public.plan_retrospective USING btree (plan_uuid);


COMMENT ON TABLE public.plan_retrospective IS 'Таблица для ретроспекции планов закупки';



COMMENT ON COLUMN public.plan_retrospective.id IS 'Уникальный идентификатор записи';



COMMENT ON COLUMN public.plan_retrospective.plan_uuid IS 'Уникальный идентификатор привязанного ППЗ';



COMMENT ON COLUMN public.plan_retrospective.plan_id IS 'Уникальный идентификатор привязанного ППЗ';



COMMENT ON COLUMN public.plan_retrospective.plan_year IS 'Год планирования исторического плана закупок';



COMMENT ON COLUMN public.plan_retrospective.plan_status IS 'Статус исторического плана закупок';



COMMENT ON COLUMN public.plan_retrospective.id_ly IS 'ID прошлого года';



COMMENT ON COLUMN public.plan_retrospective.uuid_ly IS 'UUID прошлого года';


