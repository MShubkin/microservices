-- Table: public.conclusion_templates_criteria
CREATE TABLE public.conclusion_templates_criteria (
	id SMALLSERIAL PRIMARY KEY,
	uuid UUID NOT NULL,
	template_id SMALLINT DEFAULT 0,
	template_type_id SMALLINT DEFAULT 0,
	purchasing_method_id SMALLINT[] NOT NULL DEFAULT '{0}'::smallint[],
	purchasing_trend_id SMALLINT[] NOT NULL DEFAULT '{0}'::smallint[],
	pricing_subject_purchase_id SMALLINT[] NOT NULL DEFAULT '{0}'::smallint[],
	is_removed BOOLEAN NOT NULL DEFAULT FALSE,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
	changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
	created_by INTEGER NOT NULL DEFAULT 0,
	changed_by INTEGER NOT NULL DEFAULT 0,
	FOREIGN KEY (template_id) REFERENCES public.conclusion_templates (id) ON DELETE CASCADE
);

COMMENT ON TABLE public.conclusion_templates_criteria IS 'Справочник "Критерии Шаблонов заключений"';
COMMENT ON COLUMN public.conclusion_templates_criteria.id IS 'ID Записи(генерируется автоматически)';
COMMENT ON COLUMN public.conclusion_templates_criteria.uuid IS 'UID Записи';
COMMENT ON COLUMN public.conclusion_templates_criteria.template_id IS 'ID Шаблона';
COMMENT ON COLUMN public.conclusion_templates_criteria.purchasing_method_id IS 'Ид. способа закупки';
COMMENT ON COLUMN public.conclusion_templates_criteria.purchasing_trend_id IS 'Ид. направления закупки';
COMMENT ON COLUMN public.conclusion_templates_criteria.pricing_subject_purchase_id IS 'Ид. предмета закупки ';
COMMENT ON COLUMN public.conclusion_templates_criteria.template_type_id IS 'Тип шаблона';
COMMENT ON COLUMN public.conclusion_templates_criteria.is_removed IS 'Запись удалена';
COMMENT ON COLUMN public.conclusion_templates_criteria.created_by IS 'Идентификатор создателя';
COMMENT ON COLUMN public.conclusion_templates_criteria.changed_by IS 'Идентификатор того кто изменил';
COMMENT ON COLUMN public.conclusion_templates_criteria.created_at IS 'Дата создания';
COMMENT ON COLUMN public.conclusion_templates_criteria.changed_at IS 'Дата изменения';
