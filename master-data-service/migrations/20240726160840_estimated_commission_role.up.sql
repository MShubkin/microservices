-- Table: public.estimated_commission_role
CREATE TABLE public.estimated_commission_role (
  id SMALLSERIAL PRIMARY KEY,
  name VARCHAR(250) NOT NULL DEFAULT '',
  uuid uuid NOT NULL,
  is_removed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.estimated_commission_role IS 'Справочник «Роли пользователей Сметной комиссии»';

COMMENT ON COLUMN public.estimated_commission_role.uuid IS 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.estimated_commission_role.id IS 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.estimated_commission_role.name IS 'Значения:
1 - Председатель Сметной комиссии
2 - Член Сметной комиссии
3 - Секретарь Сметной комиссии';
COMMENT ON COLUMN public.estimated_commission_role.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.estimated_commission_role.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.estimated_commission_role.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.estimated_commission_role.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';
COMMENT ON COLUMN public.estimated_commission_role.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';

INSERT INTO estimated_commission_role(name, uuid, created_at, changed_at, created_by, changed_by)
VALUES
    ('Председатель Сметной комиссии', gen_random_uuid(), now(), now(), 1, 1),
    ('Член Сметной комиссии', gen_random_uuid(), now(), now(), 1, 1),
    ('Секретарь Сметной комиссии', gen_random_uuid(), now(), now(), 1, 1)
;
