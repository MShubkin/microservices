-- Table: public.conclusion_templates_variables
CREATE TABLE public.conclusion_templates_variables (
	id SMALLSERIAL PRIMARY KEY,
	uuid UUID NOT NULL,
	template_id SMALLINT DEFAULT 0,
	template_type_id SMALLINT DEFAULT 0, -- TODO: Add conclusion_template_type table ?
	var_code VARCHAR(20) NOT NULL DEFAULT '',
	var_name VARCHAR(40) NOT NULL DEFAULT '',
	data_type VARCHAR(4) NOT NULL DEFAULT '',
	leng INTEGER DEFAULT 0,
	decimals INTEGER DEFAULT 0,
	it_name VARCHAR(16) NOT NULL DEFAULT '',
	it_field VARCHAR(30) NOT NULL DEFAULT '',
	it_formula VARCHAR(200) NOT NULL DEFAULT '',
	grp_oper SMALLINT DEFAULT 0,
	ext_data VARCHAR(200) NOT NULL DEFAULT '',
	is_removed BOOLEAN NOT NULL DEFAULT FALSE,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
	changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
	created_by INTEGER NOT NULL DEFAULT 0,
	changed_by INTEGER NOT NULL DEFAULT 0,
	FOREIGN KEY (template_id) REFERENCES public.conclusion_templates (id) ON DELETE CASCADE
);

COMMENT ON TABLE public.conclusion_templates_variables IS 'Справочник "Переменные Шаблонов заключений"';

COMMENT ON COLUMN public.conclusion_templates_variables.id IS 'ID Записи(генерируется автоматически)';
COMMENT ON COLUMN public.conclusion_templates_variables.uuid IS 'UID Записи';
COMMENT ON COLUMN public.conclusion_templates_variables.template_type_id IS 'Тип шаблона';
COMMENT ON COLUMN public.conclusion_templates_variables.var_code IS 'Код переменной';
COMMENT ON COLUMN public.conclusion_templates_variables.var_name IS 'Название переменной';
COMMENT ON COLUMN public.conclusion_templates_variables.data_type IS 'Тип данных';
COMMENT ON COLUMN public.conclusion_templates_variables.leng IS 'Длина (число знаков)';
COMMENT ON COLUMN public.conclusion_templates_variables.decimals IS 'Число десятичных разрядов';
COMMENT ON COLUMN public.conclusion_templates_variables.it_name IS 'Имя внутреннего источника';
COMMENT ON COLUMN public.conclusion_templates_variables.it_field IS 'Поле внутреннего источника';
COMMENT ON COLUMN public.conclusion_templates_variables.it_formula IS 'Формула вычисления с именами полей внутреннего источника';
COMMENT ON COLUMN public.conclusion_templates_variables.grp_oper IS 'Операция группирования (позиции)';
COMMENT ON COLUMN public.conclusion_templates_variables.ext_data IS 'Внешний источник данных';
COMMENT ON COLUMN public.conclusion_templates_variables.is_removed IS 'Запись удалена';
COMMENT ON COLUMN public.conclusion_templates_variables.created_by IS 'Идентификатор создателя';
COMMENT ON COLUMN public.conclusion_templates_variables.changed_by IS 'Идентификатор того, кто изменил';
COMMENT ON COLUMN public.conclusion_templates_variables.created_at IS 'Дата создания';
COMMENT ON COLUMN public.conclusion_templates_variables.changed_at IS 'Дата изменения';
