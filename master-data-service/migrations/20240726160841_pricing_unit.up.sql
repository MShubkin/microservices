-- Table: public.pricing_organization_unit
CREATE TABLE public.pricing_organization_unit (
  id SMALLSERIAL PRIMARY KEY,
  uuid uuid NOT NULL,
  name VARCHAR(250) NOT NULL DEFAULT '',
  sap_code INTEGER NOT NULL DEFAULT 0,
  is_removed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  created_by INTEGER NOT NULL,
  changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.pricing_organization_unit IS 'Справочник "Департаменты (Организации) АЦ"';
COMMENT ON COLUMN public.pricing_organization_unit.uuid IS 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.pricing_organization_unit.id IS 'Значения:
5* - Д646
5* - Д647
5* - ГПК
';
COMMENT ON COLUMN public.pricing_organization_unit.sap_code IS ' Код орг. структуры SAP';
COMMENT ON COLUMN public.pricing_organization_unit.name IS 'Значения:
5* - Д646
5* - Д647
5* - ГПК
';

COMMENT ON COLUMN public.pricing_organization_unit.is_removed IS 'Признак удаления записи';

COMMENT ON COLUMN public.pricing_organization_unit.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.pricing_organization_unit.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.pricing_organization_unit.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.pricing_organization_unit.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';
