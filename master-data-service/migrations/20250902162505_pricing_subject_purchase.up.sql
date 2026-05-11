CREATE TABLE public.pricing_subject_purchase (
    uuid uuid NOT NULL PRIMARY KEY,
    id INTEGER NOT NULL,
    text VARCHAR(100) NOT NULL,
    pricing_organization_unit_id SMALLINT NOT NULL,
    purchasing_trend_id SMALLINT NOT NULL,
    access_id SMALLINT DEFAULT 1 NOT NULL,
    hierarchy_id INTEGER NOT NULL,
    hierarchy_uuid uuid NOT NULL,
    parent_uuid uuid,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_by INTEGER NOT NULL,
    is_removed BOOLEAN DEFAULT false NOT NULL
);

COMMENT ON TABLE public.pricing_subject_purchase IS 'Предметы/Группы Предметов закупки АЦ';

COMMENT ON COLUMN public.pricing_subject_purchase.uuid IS 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.pricing_subject_purchase.id IS 'Идентификатор предмета закупки/группы предмета закупки';
COMMENT ON COLUMN public.pricing_subject_purchase.text IS 'Наименование предмета закупки/группы предмета';
COMMENT ON COLUMN public.pricing_subject_purchase.pricing_organization_unit_id IS 'Орг.единица АЦ';
COMMENT ON COLUMN public.pricing_subject_purchase.purchasing_trend_id IS 'ID направления закупки';
COMMENT ON COLUMN public.pricing_subject_purchase.access_id IS 'Доступ к записи';
COMMENT ON COLUMN public.pricing_subject_purchase.hierarchy_id IS 'Уровень иерархии';
COMMENT ON COLUMN public.pricing_subject_purchase.hierarchy_uuid IS 'Уникальный идентификатор вышестоящей записи';
COMMENT ON COLUMN public.pricing_subject_purchase.parent_uuid IS 'Уникальный идентификатор родительской записи';

COMMENT ON COLUMN public.pricing_subject_purchase.created_by IS 'Автор создания';
COMMENT ON COLUMN public.pricing_subject_purchase.created_at IS 'Дата и время создания';
COMMENT ON COLUMN public.pricing_subject_purchase.changed_by IS 'Автор изменения';
COMMENT ON COLUMN public.pricing_subject_purchase.changed_at IS 'Дата и время изменения';
COMMENT ON COLUMN public.pricing_subject_purchase.is_removed IS 'Признак удаления';
