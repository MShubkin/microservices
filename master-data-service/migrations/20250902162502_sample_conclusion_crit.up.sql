DROP TABLE IF EXISTS public.crit_sample_conclusion;

CREATE TABLE public.sample_conclusion_crit (
    sample_conclusion_uuid uuid NOT NULL PRIMARY KEY,
    field_name VARCHAR(60) NOT NULL,
    predicate JSONB NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_by INTEGER NOT NULL,
    is_removed BOOLEAN DEFAULT false NOT NULL
);

COMMENT ON TABLE public.sample_conclusion_crit IS 'Шаблоны заключений';

COMMENT ON COLUMN public.sample_conclusion_crit.sample_conclusion_uuid IS 'Уникальный идентификатор';
COMMENT ON COLUMN public.sample_conclusion_crit.field_name IS 'Название критерия';
COMMENT ON COLUMN public.sample_conclusion_crit.predicate IS 'Значение критерия';

COMMENT ON COLUMN public.sample_conclusion_crit.created_by IS 'Автор создания';
COMMENT ON COLUMN public.sample_conclusion_crit.created_at IS 'Дата и время создания';
COMMENT ON COLUMN public.sample_conclusion_crit.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.sample_conclusion_crit.changed_at IS 'Дата и время изменения';
COMMENT ON COLUMN public.sample_conclusion_crit.is_removed IS 'Признак удаления';
