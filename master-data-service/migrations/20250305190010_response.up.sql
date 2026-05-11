CREATE TABLE public.response (
    id smallint NOT NULL,
    text character varying(256) NOT NULL,
    color_code character(6) NOT NULL,
    note_obligation smallint NOT NULL,
    sap_id character(20) NOT NULL,
    is_removed boolean NOT NULL,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    created_by integer NOT NULL,
    changed_by integer NOT NULL,
    icon text
);

ALTER TABLE ONLY public.response
    ADD CONSTRAINT response_pkey PRIMARY KEY (id);

COMMENT ON TABLE public.response IS 'Справочник решений в базе данных НСИ';

COMMENT ON COLUMN public.response.id IS 'id Решения Эксперта ПД';
COMMENT ON COLUMN public.response.text IS 'Текст решения';
COMMENT ON COLUMN public.response.color_code IS 'Код цвета в HEX-формате';
COMMENT ON COLUMN public.response.note_obligation IS 'Обязательность комментария к решению (1 - не проверяется, 2 - обязательно к заполнению, 3 - запрет заполнения)';
COMMENT ON COLUMN public.response.sap_id IS 'Код в САП - АСЭЗ (для синхронизации)';
COMMENT ON COLUMN public.response.is_removed IS 'Индикатор удаления записи';
COMMENT ON COLUMN public.response.created_at IS 'Дата и время создания записи';
COMMENT ON COLUMN public.response.changed_at IS 'id пользователя, создавшего запись';
COMMENT ON COLUMN public.response.created_by IS 'Дата и время последнего изменения записи';
COMMENT ON COLUMN public.response.changed_by IS 'id пользователя, изменившего запись';
