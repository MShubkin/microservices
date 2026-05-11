DROP TABLE IF EXISTS public.list_sample_conclusion;

CREATE TABLE public.sample_conclusion (
    uuid uuid NOT NULL PRIMARY KEY,
    id SERIAL NOT NULL,
    text VARCHAR(120)  NOT NULL,
    access_id SMALLINT DEFAULT 1 NOT NULL,
    status_id SMALLINT NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_by INTEGER NOT NULL,
    is_removed BOOLEAN DEFAULT false NOT NULL
);

COMMENT ON TABLE public.sample_conclusion IS 'Шаблоны заключений';

COMMENT ON COLUMN public.sample_conclusion.uuid IS 'Идентификатор шаблона заключения';
COMMENT ON COLUMN public.sample_conclusion.id IS 'Системный номер шаблона';
COMMENT ON COLUMN public.sample_conclusion.text IS 'Текст шаблона заключения';
COMMENT ON COLUMN public.sample_conclusion.access_id IS 'Доступ к шаблону заключения';
COMMENT ON COLUMN public.sample_conclusion.status_id IS 'Статус шаблона';

COMMENT ON COLUMN public.sample_conclusion.created_by IS 'Автор создания';
COMMENT ON COLUMN public.sample_conclusion.created_at IS 'Дата и время создания';
COMMENT ON COLUMN public.sample_conclusion.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.sample_conclusion.changed_at IS 'Дата и время изменения';
COMMENT ON COLUMN public.sample_conclusion.is_removed IS 'Признак удаления';
