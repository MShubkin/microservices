-- Table: public.payment_conditions
CREATE TABLE public.payment_conditions
(
	id SMALLSERIAL PRIMARY KEY,
	uuid uuid NOT NULL,
	name VARCHAR(250) NOT NULL DEFAULT '',
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.payment_conditions IS 'Справочник "Условия оплаты"';

COMMENT ON COLUMN public.payment_conditions.uuid IS 'UID Записи';
COMMENT ON COLUMN public.payment_conditions.id IS 'ID Записи';
COMMENT ON COLUMN public.payment_conditions.name IS 'Наименование';
COMMENT ON COLUMN public.payment_conditions.changed_at IS 'Дата изменения';

COMMENT ON COLUMN public.payment_conditions.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.payment_conditions.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.payment_conditions.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.payment_conditions.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

INSERT INTO public.payment_conditions(uuid, name, changed_at, created_at, changed_by, created_by)
	VALUES
	('26c94049-6316-4d75-bb79-014528831b44', 'Аванс', now(), now(), 1, 1),
	('e48f69a3-9f4d-4cbf-9a4f-2289b2393b5c', 'Постоплата', now(), now(), 1, 1)
;
