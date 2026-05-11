-- Table: public.conclusion_templates
CREATE TABLE public.conclusion_templates (
  id SMALLSERIAL PRIMARY KEY,
  uuid UUID NOT NULL,
  template_type_id SMALLINT DEFAULT 0,
  pricing_unit_id SMALLINT DEFAULT 0,
  access_id SMALLINT DEFAULT 1,
  status_id SMALLINT DEFAULT 0,
  text VARCHAR(1000) NOT NULL DEFAULT '',
  is_removed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  created_by INTEGER NOT NULL DEFAULT 0,
  changed_by INTEGER NOT NULL DEFAULT 0
);

COMMENT ON TABLE public.conclusion_templates IS 'Справочник "Шаблоны заключений"';

COMMENT ON COLUMN public.conclusion_templates.id IS 'ID Шаблона(генерируется автоматически)';
COMMENT ON COLUMN public.conclusion_templates.uuid IS 'UID Записи';
COMMENT ON COLUMN public.conclusion_templates.template_type_id IS 'Тип шаблона';
COMMENT ON COLUMN public.conclusion_templates.pricing_unit_id IS 'Департамент (Организация) АЦ';
COMMENT ON COLUMN public.conclusion_templates.access_id IS 'Доступ к шаблону заключения';
COMMENT ON COLUMN public.conclusion_templates.status_id IS 'Статус шаблона заключения';
COMMENT ON COLUMN public.conclusion_templates.text IS 'Текст шаблона';
COMMENT ON COLUMN public.conclusion_templates.is_removed IS 'Запись удалена';
COMMENT ON COLUMN public.conclusion_templates.created_by IS 'Идентификатор создателя';
COMMENT ON COLUMN public.conclusion_templates.changed_by IS 'Идентификатор того, кто изменил';
COMMENT ON COLUMN public.conclusion_templates.created_at IS 'Дата создания';
COMMENT ON COLUMN public.conclusion_templates.changed_at IS 'Дата изменения';
