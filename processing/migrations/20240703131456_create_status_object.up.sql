

-- This table is will probably not be used as we will
-- likely just use a hardcoded enum.
CREATE TABLE public.status_object(
	id SMALLINT NOT NULL PRIMARY KEY UNIQUE,
	"value" VARCHAR(50) NOT NULL,
	created_at timestamp without time zone NOT NULL,
	changed_at timestamp without time zone NOT NULL,
	created_by INTEGER NOT NULL,
	changed_by INTEGER NOT NULL
) TABLESPACE pg_default;

COMMENT ON TABLE public.status_object IS 'Таблица справочник статусов объекта';

COMMENT ON COLUMN public.status_object.id IS 'Уникальный идентификатор записи';

COMMENT ON COLUMN public.status_object."value" IS 'Описание статуса объекта';

COMMENT ON COLUMN public.status_object.created_by IS 'Идентификатор создателя';

COMMENT ON COLUMN public.status_object.changed_by IS 'Идентификатор того кто изменил';

COMMENT ON COLUMN public.status_object.created_at IS 'Дата создания';

COMMENT ON COLUMN public.status_object.changed_at IS 'Дата изменения';

INSERT INTO
	public.status_object(
		id,
		"value",
		created_at,
		changed_at,
		created_by,
		changed_by
	)
values
	(1, 'Сформирован', now()::timestamp without time zone, now()::timestamp without time zone, 0, 0),
	(2, 'На согласовании', now()::timestamp without time zone, now()::timestamp without time zone, 0, 0),
	(3, 'На подписании', now()::timestamp without time zone, now()::timestamp without time zone, 0, 0),
	(4, 'Утвержден', now()::timestamp without time zone, now()::timestamp without time zone, 0, 0),
	(5, 'Удален', now()::timestamp without time zone, now()::timestamp without time zone, 0, 0);
