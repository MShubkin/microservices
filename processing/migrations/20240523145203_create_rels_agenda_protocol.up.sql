

CREATE TABLE public.agenda_protocol_relation (
  protocol_uuid uuid NOT NULL,
  agenda_uuid uuid NOT NULL,
  created_at timestamp without time zone NOT NULL,
  created_by INTEGER NOT NULL,
  CONSTRAINT agenda_protocol_relation_pkey PRIMARY KEY (protocol_uuid, agenda_uuid)
) TABLESPACE pg_default;

COMMENT ON TABLE public.agenda_protocol_relation IS 'Связь между протоколом и повесткой';

COMMENT ON COLUMN public.agenda_protocol_relation.agenda_uuid IS 'Уникальный идентификатор Повестки';

COMMENT ON COLUMN public.agenda_protocol_relation.protocol_uuid IS 'Уникальный идентификатор Протокола';

COMMENT ON COLUMN public.agenda_protocol_relation.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

COMMENT ON COLUMN public.agenda_protocol_relation.created_at IS 'Дата и время создания.
Автоматически присваивается при создании объекта';
