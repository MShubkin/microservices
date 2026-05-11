CREATE TABLE public.plan_reason_cancel_check_reason (
    id SMALLINT PRIMARY KEY,
    text VARCHAR(120) DEFAULT ''::VARCHAR NOT NULL,
    is_removed BOOLEAN DEFAULT false NOT NULL,
    created_by INTEGER DEFAULT 0 NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT NOW() NOT NULL,
    changed_by INTEGER DEFAULT 0 NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE DEFAULT NOW() NOT NULL
);

INSERT INTO public.plan_reason_cancel_check_reason (id, text) VALUES 
(1, 'Публикация ППЗ в ЕИС'),
(2, 'Новая ППЗ с признаком "Прейскурасная закупка"'),
(3, 'Протокол очной СК');

COMMENT ON TABLE public.plan_reason_cancel_check_reason IS 'Справочник «Проверки для ППЗ»';
COMMENT ON COLUMN public.plan_reason_cancel_check_reason.id IS 'Идентификатор';
COMMENT ON COLUMN public.plan_reason_cancel_check_reason.text IS 'Наименование';
COMMENT ON COLUMN public.plan_reason_cancel_check_reason.is_removed IS 'Признак удаления';
COMMENT ON COLUMN public.plan_reason_cancel_check_reason.created_by IS 'Автор создания';
COMMENT ON COLUMN public.plan_reason_cancel_check_reason.created_at IS 'Дата и время создания';
COMMENT ON COLUMN public.plan_reason_cancel_check_reason.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.plan_reason_cancel_check_reason.changed_at IS 'Дата и время изменения';