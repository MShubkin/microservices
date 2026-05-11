-- Table: public.object_type
CREATE TABLE public.object_type (
    id SMALLSERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL DEFAULT '',
    code VARCHAR(50) NOT NULL DEFAULT '',
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.object_type IS 'Справочник "Типы объектов"';

COMMENT ON COLUMN public.object_type.id IS 'Значения:
1 - План
2 - Контракт/Исправление';

COMMENT ON COLUMN public.object_type.name IS 'Наименование типа объекта
1 - План
2 - Контракт/Исправление';

COMMENT ON COLUMN public.object_type.code IS 'Код типа объекта
1 - plan
2 - contract_amendment';

COMMENT ON COLUMN public.object_type.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.object_type.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.object_type.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.object_type.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.object_type.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

INSERT INTO public.object_type VALUES 
    (1, 'План', 'plan', false, '2025-04-16 05:03:48.750033', '2025-04-16 05:03:48.750033', 1, 1),
    (2, 'ДС', 'contract_amendment', false, '2025-04-16 05:03:48.750033', '2025-04-16 05:03:48.750033', 1, 1),
    (3, 'Повестка', 'agenda', false, '2025-04-16 05:03:48.750033', '2025-04-16 05:03:48.750033', 1, 1),
    (4, 'Протокол', 'protocol', false, '2025-04-16 05:03:48.750033', '2025-04-16 05:03:48.750033', 1, 1),
    (5, 'ЗП', 'purchase', false, '2025-04-16 05:03:48.750033', '2025-04-16 05:03:48.750033', 1, 1),
    (6, 'ВП', 'quotation', false, '2025-04-16 05:03:48.750033', '2025-04-16 05:03:48.750033', 1, 1),
    (7, 'КД', 'contract', false, '2025-04-16 05:03:48.750033', '2025-04-16 05:03:48.750033', 1, 1);
