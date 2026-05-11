CREATE TABLE public.sample_conclusion_status_history (
    uuid uuid NOT NULL,
    sample_conclusion_uuid uuid NOT NULL,
    status_id INTEGER NOT NULL,
    comment VARCHAR(60),
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL
);

COMMENT ON TABLE public.sample_conclusion_status_history IS 'История изменения статуса шаблона заключения';

COMMENT ON COLUMN public.sample_conclusion_status_history.uuid IS 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.sample_conclusion_status_history.sample_conclusion_uuid IS 'Уникальный идентификатор шаблона заключения';
COMMENT ON COLUMN public.sample_conclusion_status_history.status_id IS 'Статус шаблона заключения';
COMMENT ON COLUMN public.sample_conclusion_status_history.comment IS 'Комментарий';

COMMENT ON COLUMN public.sample_conclusion_status_history.created_at IS 'Дата создания';
COMMENT ON COLUMN public.sample_conclusion_status_history.created_by IS 'Автор создания';
