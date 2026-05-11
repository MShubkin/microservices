CREATE TABLE public.output_form(
    id smallint PRIMARY KEY,
    -- Use "ObjectType", 3=Agenda, 4=Protocol.
    object_type SMALLINT NOT NULL,
    code VARCHAR(256),
    "text" VARCHAR(256),
    is_removed BOOLEAN NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
);

-- "IF EXISTS" значит там уже может быть разнородная чуж.
DELETE FROM public.output_form;
INSERT INTO public.output_form
    (id, object_type, code, "text", is_removed, created_at, changed_at, created_by, changed_by)
VALUES
    (1,  4,   '2', 'Протокол СК (заочный)'                , false, now(), now(), -1, -1),
    (2,  4,   '3', 'Бюллетень СК (заочный)'               , false, now(), now(), -1, -1),
    (3,  4,   '4', 'Приложение № 1 Работы'                , false, now(), now(), -1, -1),
    (4,  3, '5.1', 'Приглашение Департаменты'             , false, now(), now(), -1, -1),
    (5,  3, '5.2', 'Приглашение СКЗ'                      , false, now(), now(), -1, -1),
    (6,  3, '5.3', 'Приглашение Реестр общий (приложение)', false, now(), now(), -1, -1),
    (7,  3, '5.4', 'Приглашение ДО-заказчики'             , false, now(), now(), -1, -1),
    (8,  3, '5.5', 'Приглашение ГСП'                      , false, now(), now(), -1, -1),
    (9,  3, '5.6', 'Реестр ГСП (приложение)'              , false, now(), now(), -1, -1),
    (10, 3, '5.7', 'Приглашение ГИнвест-заказчик'         , false, now(), now(), -1, -1),
    (11, 3, '5.8', 'Реестр ГИнвест (приложение)'          , false, now(), now(), -1, -1),
    (12, 3,   '6', 'Реестр СК (Повестка)'                 , false, now(), now(), -1, -1),
    (13, 4,   '7', 'Выписка'                              , false, now(), now(), -1, -1);

COMMENT ON TABLE public.output_form IS 'Справочник "Выходная форма" (output_form).';

COMMENT ON COLUMN public.output_form.id IS 'ID типа формы (form_type_id)';
COMMENT ON COLUMN public.output_form.object_type IS 'Тип объекта:
 - agenda (Повестка)
 - protocol (Протокол)

*Протокол может быть очной СК и заочной СК';
COMMENT ON COLUMN public.output_form.code IS 'Код формы, используемый заказчиком. Необязательное поле. Может быть пустым';
COMMENT ON COLUMN public.output_form.text IS 'Наименование формируемого файла';
COMMENT ON COLUMN public.output_form.is_removed IS '';
COMMENT ON COLUMN public.output_form.created_at IS 'Дата и время создания.';
COMMENT ON COLUMN public.output_form.created_by IS 'Логин пользователя ответственного исполнителя, создавшего запись';
COMMENT ON COLUMN public.output_form.changed_at IS 'Дата и время изменения.';
COMMENT ON COLUMN public.output_form.changed_by IS 'Логин пользователя ответственного исполнителя, изменившего запись';
