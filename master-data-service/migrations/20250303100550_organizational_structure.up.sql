CREATE TABLE public.organizational_structure (
    uuid uuid NOT NULL,
    id integer DEFAULT 0 NOT NULL,
    text character varying(50) DEFAULT ''::character varying NOT NULL,
    text_short character varying(20) DEFAULT ''::character varying NOT NULL,
    level smallint DEFAULT 1 NOT NULL,
    parent_id integer,
    type smallint DEFAULT 1 NOT NULL,
    is_specialized_department boolean DEFAULT false NOT NULL,
    sap_id integer,
    is_removed boolean DEFAULT false NOT NULL,
    created_at timestamp without time zone DEFAULT '1900-01-01 00:00:00'::timestamp without time zone NOT NULL,
    changed_at timestamp without time zone DEFAULT '1900-01-01 00:00:00'::timestamp without time zone NOT NULL,
    created_by integer DEFAULT 0 NOT NULL,
    changed_by integer DEFAULT 0 NOT NULL
);

ALTER TABLE ONLY public.organizational_structure
    ADD CONSTRAINT organizational_structure_pkey PRIMARY KEY (uuid);

COMMENT ON TABLE public.organizational_structure IS 'Департаменты и подразделения (орг. план)';

COMMENT ON COLUMN public.organizational_structure.uuid IS 'гуид записи справочника Department';
COMMENT ON COLUMN public.organizational_structure.id IS 'id подразделения';
COMMENT ON COLUMN public.organizational_structure.text IS 'Длинное название подразделения';
COMMENT ON COLUMN public.organizational_structure.text_short IS 'Короткое название подразделения';
COMMENT ON COLUMN public.organizational_structure.level IS 'уровень в иерархии';
COMMENT ON COLUMN public.organizational_structure.parent_id IS 'id родительского  подразделения   1-го уровня (ПАО)';
COMMENT ON COLUMN public.organizational_structure.type IS 'Тип подразделения';
COMMENT ON COLUMN public.organizational_structure.is_specialized_department IS 'флаг использования в модуле ПД';
COMMENT ON COLUMN public.organizational_structure.sap_id IS 'Код в оргструктуре САП - АСЭЗ (для синхронизации)';
COMMENT ON COLUMN public.organizational_structure.is_removed IS 'флаг удаления/деактивации';
COMMENT ON COLUMN public.organizational_structure.created_at IS 'id пользователя изменившего запись';
COMMENT ON COLUMN public.organizational_structure.changed_at IS 'дата+время последнего изменения';
COMMENT ON COLUMN public.organizational_structure.created_by IS 'id пользователя создавшего запись';
