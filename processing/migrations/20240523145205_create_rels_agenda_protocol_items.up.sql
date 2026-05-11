

CREATE TABLE public.item_relation_agenda_protocol (
  agenda_item_uuid uuid NOT NULL,
  agenda_uuid uuid NOT NULL,
  protocol_uuid uuid NOT NULL,
  protocol_item_uuid uuid NOT NULL,
  created_at timestamp without time zone NOT NULL,
  created_by INTEGER NOT NULL,
  -- Здесь достаточно protocol_uuid_item, agenda_uuid_item, так как они
  -- относятся к одной Повестке и одному Протоколу
  CONSTRAINT "item_relation_agenda_protocol_pkey" PRIMARY KEY (protocol_item_uuid, agenda_item_uuid)
) TABLESPACE pg_default;

COMMENT ON TABLE public.item_relation_agenda_protocol IS 'Связь между protocol_item и agenda_item.';

COMMENT ON COLUMN public.item_relation_agenda_protocol.agenda_uuid IS 'Уникальный идентификатор Повестки';

COMMENT ON COLUMN public.item_relation_agenda_protocol.protocol_uuid IS 'Уникальный идентификатор Протокола';

COMMENT ON COLUMN public.item_relation_agenda_protocol.agenda_item_uuid IS 'Уникальный идентификатор записи позиции в Повестке';

COMMENT ON COLUMN public.item_relation_agenda_protocol.protocol_item_uuid IS 'Уникальный идентификатор записи позиции в Протоколе';

COMMENT ON COLUMN public.item_relation_agenda_protocol.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

COMMENT ON COLUMN public.item_relation_agenda_protocol.created_at IS 'Дата и время создания.
Автоматически присваивается при создании объекта';
