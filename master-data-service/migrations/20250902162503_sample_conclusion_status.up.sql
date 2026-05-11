CREATE TABLE  public.sample_conclusion_status (
    id SMALLINT NOT NULL,
    text VARCHAR(60) NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_by INTEGER NOT NULL,
    is_removed BOOLEAN DEFAULT false NOT NULL
);

COMMENT ON TABLE public.sample_conclusion_status IS 'Статусы шаблона заключения';

COMMENT ON COLUMN public.sample_conclusion_status.id IS 'Идентификатор статуса шаблона';
COMMENT ON COLUMN public.sample_conclusion_status.text IS 'Наименование статуса шаблона';

COMMENT ON COLUMN public.sample_conclusion_status.created_by IS 'Автор создания';
COMMENT ON COLUMN public.sample_conclusion_status.created_at IS 'Дата и время создания';
COMMENT ON COLUMN public.sample_conclusion_status.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.sample_conclusion_status.changed_at IS 'Дата и время изменения';
COMMENT ON COLUMN public.sample_conclusion_status.is_removed IS 'Признак удаления';
