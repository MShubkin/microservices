CREATE TABLE public.plan_reason_cancel (
    id SERIAL PRIMARY KEY,
    text VARCHAR(120) DEFAULT ''::VARCHAR NOT NULL,
    impact_area_id SMALLINT NOT NULL,
    is_objective_reason BOOLEAN DEFAULT false NOT NULL,
    is_new_plan BOOLEAN DEFAULT false NOT NULL,
    is_removed BOOLEAN DEFAULT false NOT NULL,
    is_reason_fill_type BOOLEAN DEFAULT false NOT NULL,
    functionality_id_list smallint[] DEFAULT '{}' NOT NULL,
    check_reason_id smallint[] DEFAULT '{}' NOT NULL,
    created_by INTEGER DEFAULT 0 NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT '1900-01-01 00:00:00'::TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_by INTEGER DEFAULT 0 NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE DEFAULT '1900-01-01 00:00:00'::TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

COMMENT ON TABLE public.plan_reason_cancel IS 'Справочник «Причины аннулирования закупки»';
COMMENT ON COLUMN public.plan_reason_cancel.id IS 'Идентификатор записи причины аннулирования';
COMMENT ON COLUMN public.plan_reason_cancel.text IS 'Наименование причины';
COMMENT ON COLUMN public.plan_reason_cancel.impact_area_id IS 'Основание аннулирования';
COMMENT ON COLUMN public.plan_reason_cancel.is_objective_reason IS 'Объективная причина';
COMMENT ON COLUMN public.plan_reason_cancel.is_new_plan IS 'Новая ППЗ/ДС';
COMMENT ON COLUMN public.plan_reason_cancel.is_removed IS 'Признак удаления';
COMMENT ON COLUMN public.plan_reason_cancel.is_reason_fill_type IS 'Автоматическое заполнение причины';
COMMENT ON COLUMN public.plan_reason_cancel.functionality_id_list IS 'Функциональность';
COMMENT ON COLUMN public.plan_reason_cancel.created_by IS 'Автор создания';
COMMENT ON COLUMN public.plan_reason_cancel.created_at IS 'Дата и время создания';
COMMENT ON COLUMN public.plan_reason_cancel.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.plan_reason_cancel.changed_at IS 'Дата и время изменения';

CREATE TABLE public.plan_reason_customer (
    id SERIAL PRIMARY KEY,
    plan_reason_cancel_id INTEGER DEFAULT 0 NOT NULL,
    customer_id INTEGER DEFAULT 0 NOT NULL,
    is_removed BOOLEAN DEFAULT false NOT NULL, 
    created_by INTEGER DEFAULT 0 NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT '1900-01-01 00:00:00'::TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_by INTEGER DEFAULT 0 NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE DEFAULT '1900-01-01 00:00:00'::TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    
    CONSTRAINT uniq_plan_reason_customer 
        UNIQUE (plan_reason_cancel_id, customer_id),
    
    CONSTRAINT fk_plan_reason_cancel
        FOREIGN KEY(plan_reason_cancel_id) REFERENCES public.plan_reason_cancel(id) ON DELETE CASCADE
);

COMMENT ON TABLE public.plan_reason_customer IS 'Заказчики для причин аннулирования';
COMMENT ON COLUMN public.plan_reason_customer.id IS 'Идентификатор записи';
COMMENT ON COLUMN public.plan_reason_customer.plan_reason_cancel_id IS 'Идентификатор записи причины аннулирования';
COMMENT ON COLUMN public.plan_reason_customer.customer_id IS 'Заказчик';
COMMENT ON COLUMN public.plan_reason_customer.is_removed IS 'Признак удаления';
COMMENT ON COLUMN public.plan_reason_customer.created_by IS 'Автор создания';
COMMENT ON COLUMN public.plan_reason_customer.created_at IS 'Дата и время создания';
COMMENT ON COLUMN public.plan_reason_customer.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.plan_reason_customer.changed_at IS 'Дата и время изменения';