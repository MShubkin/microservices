CREATE TABLE public.plan_reason_cancel_impact_area (
    id SMALLINT PRIMARY KEY,
    text VARCHAR(120) DEFAULT ''::VARCHAR NOT NULL,
    is_removed BOOLEAN DEFAULT false NOT NULL,
    created_by INTEGER DEFAULT 0 NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT NOW() NOT NULL,
    changed_by INTEGER DEFAULT 0 NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE DEFAULT NOW() NOT NULL
);

INSERT INTO public.plan_reason_cancel_impact_area (id, text) VALUES 
(1, 'Сфера влияния КГГ'),
(2, 'Сфера влияния ПАО Газпром'),
(3, 'Технические причины'),
(4, 'Для централизованного поставщика СК');

COMMENT ON TABLE public.plan_reason_cancel_impact_area IS 'Справочник «Основания аннулирования»';
COMMENT ON COLUMN public.plan_reason_cancel_impact_area.id iS 'Идентификатор основания аннулирования';
COMMENT ON COLUMN public.plan_reason_cancel_impact_area.text IS 'Наименование основания аннулирования';
COMMENT ON COLUMN public.plan_reason_cancel_impact_area.is_removed IS 'Признак удаления';
COMMENT ON COLUMN public.plan_reason_cancel_impact_area.created_by IS 'Автор создания';
COMMENT ON COLUMN public.plan_reason_cancel_impact_area.created_at IS 'Дата и время создания';
COMMENT ON COLUMN public.plan_reason_cancel_impact_area.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.plan_reason_cancel_impact_area.changed_at IS 'Дата и время изменения';