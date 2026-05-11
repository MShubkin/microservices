

-- Probably used as: SELECT (agenda, partner) FROM agenda JOIN partner on agenda.uuid==partner.item_uuid WHERE agenda.uuid=$1;
-- or as: SELECT (protocol, partner) FROM protocol JOIN partner on protocol.uuid==partner.item_uuid WHERE protocol.uuid=$1;
-- Thus we only need one item_uuid field.
-- However, if going by WHERE partner.some_field=$1, this is not as convenient.
-- (agenda/protocol table names abbreviated here from agenda..)
CREATE TABLE public.partner_agenda_protocol (
	uuid uuid NOT NULL PRIMARY KEY,
	item_uuid uuid NOT NULL,
	-- TODO: Determine whether this is optimal.
	user_id INTEGER NOT NULL,
	user_email VARCHAR(200) NOT NULL,
	-- Probably not going to get a longer email address.
	is_present BOOLEAN NOT NULL,
	is_removed BOOLEAN NOT NULL,
	created_at timestamp without time zone NOT NULL,
	changed_at timestamp without time zone NOT NULL,
	created_by INTEGER NOT NULL,
	changed_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.partner_agenda_protocol IS 'Таблица предметов повестки';

COMMENT ON COLUMN public.partner_agenda_protocol.uuid IS 'Уникальный идентификатор записи';

COMMENT ON COLUMN public.partner_agenda_protocol.created_by IS 'Идентификатор создателя';

COMMENT ON COLUMN public.partner_agenda_protocol.changed_by IS 'Идентификатор того кто изменил';

COMMENT ON COLUMN public.partner_agenda_protocol.created_at IS 'Дата создания';

COMMENT ON COLUMN public.partner_agenda_protocol.changed_at IS 'Дата изменения';
