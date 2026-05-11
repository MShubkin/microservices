CREATE TABLE public.plan_reason_cancel_functionality (
    id SMALLINT PRIMARY KEY,
    text VARCHAR(120) DEFAULT ''::VARCHAR NOT NULL,
    is_removed BOOLEAN DEFAULT false NOT NULL,
    created_by INTEGER DEFAULT 0 NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT NOW() NOT NULL,
    changed_by INTEGER DEFAULT 0 NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE DEFAULT NOW() NOT NULL
);

INSERT INTO public.plan_reason_cancel_functionality (id, text) VALUES 
(1, 'Планирование/ППЗ'),
(2, 'Планирование/ППЗ/контрольная проверка'),
(3, 'Сметная коммиссия');

COMMENT ON TABLE public.plan_reason_cancel_functionality IS 'Справочник «Функциональность»';
COMMENT ON COLUMN public.plan_reason_cancel_functionality.id IS 'Идентификатор';
COMMENT ON COLUMN public.plan_reason_cancel_functionality.text IS 'Наименование';
COMMENT ON COLUMN public.plan_reason_cancel_functionality.is_removed IS 'Признак удаления';
COMMENT ON COLUMN public.plan_reason_cancel_functionality.created_by IS 'Автор создания';
COMMENT ON COLUMN public.plan_reason_cancel_functionality.created_at IS 'Дата и время создания';
COMMENT ON COLUMN public.plan_reason_cancel_functionality.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.plan_reason_cancel_functionality.changed_at IS 'Дата и время изменения';