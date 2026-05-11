-- Table: public.organizational_user_assignment

CREATE TABLE public.organizational_user_assignment(
    uuid UUID NOT NULL PRIMARY KEY UNIQUE, -- ??? в монолите нет
    user_id INTEGER NOT NULL,
    customer_id INTEGER,
    department_id INTEGER,
    position_id INTEGER,
    organizer_id INTEGER,
    purchasing_group_id INTEGER,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT '1900-01-01 00:00:00',
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT '1900-01-01 00:00:00',
    created_by INTEGER NOT NULL DEFAULT 0,
    changed_by INTEGER NOT NULL DEFAULT 0
);

COMMENT ON TABLE public.organizational_user_assignment IS 'Присвоение пользователя подразделению (орг. присвоение пользователя)';
COMMENT ON COLUMN public.organizational_user_assignment.uuid IS 'гуид записи таблицы (первичный ключ)';
COMMENT ON COLUMN public.organizational_user_assignment.user_id IS 'id Пользователя';
COMMENT ON COLUMN public.organizational_user_assignment.customer_id IS 'id Заказчика, к которому относится пользователь';
COMMENT ON COLUMN public.organizational_user_assignment.department_id IS 'id Подразделение, к которому относится пользователь (из справочника department)';
COMMENT ON COLUMN public.organizational_user_assignment.position_id IS 'id Должности(?) , которую занимает пользователь  (не известно)';
COMMENT ON COLUMN public.organizational_user_assignment.organizer_id IS 'id Организатора закупки (? Или Закупочной организации? ), к которому относится пользователь (не известно)';
COMMENT ON COLUMN public.organizational_user_assignment.purchasing_group_id IS 'id Группы закупок, к которому относится пользователь';
COMMENT ON COLUMN public.organizational_user_assignment.created_at IS 'дата+время создания';
COMMENT ON COLUMN public.organizational_user_assignment.changed_at IS 'дата+время последнего изменения';
COMMENT ON COLUMN public.organizational_user_assignment.created_by IS 'id пользователя создавшего запись';
COMMENT ON COLUMN public.organizational_user_assignment.created_at IS 'id пользователя изменившего запись';
