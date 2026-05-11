

CREATE TABLE public.price_analysis_user (
  id                SERIAL PRIMARY KEY,
  pricing_organization_unit_id SMALLINT NOT NULL DEFAULT 0,
  subdivision_id    SMALLINT,
  type_user_id      SMALLINT NOT NULL DEFAULT 0,
  ppz_type_id       SMALLINT NOT NULL DEFAULT 0,
  user_id           INTEGER NOT NULL DEFAULT 0,
  env_type_id       SMALLINT NOT NULL DEFAULT 0,
  start_date        TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  end_date          TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  is_removed        BOOLEAN NOT NULL DEFAULT FALSE,
  created_at        TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  changed_at        TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  created_by        INTEGER NOT NULL,
  changed_by        INTEGER NOT NULL
) TABLESPACE pg_default;

CREATE INDEX IF NOT EXISTS price_analysis_user_pricing_organization_unit_id_ix ON public.price_analysis_user(pricing_organization_unit_id);
CREATE INDEX IF NOT EXISTS price_analysis_user_pricing_expert_id_ix ON public.price_analysis_user(user_id);
CREATE INDEX IF NOT EXISTS price_analysis_user_period_ix ON public.price_analysis_user(start_date, end_date);

CREATE UNIQUE INDEX IF NOT EXISTS price_analysis_user_unique_ix ON public.price_analysis_user(
  pricing_organization_unit_id, subdivision_id, type_user_id, ppz_type_id, user_id, start_date, is_removed
);

COMMENT ON TABLE public.price_analysis_user IS 'Таблица пользователей модуля АЦ';

COMMENT ON COLUMN public.price_analysis_user.pricing_organization_unit_id IS 'Идентификатор организационной единицы';
COMMENT ON COLUMN public.price_analysis_user.subdivision_id IS 'Идентификатор подразделения';
COMMENT ON COLUMN public.price_analysis_user.type_user_id IS '
  Тип пользователя: 
  1 - Руководитель АЦ
  2 - Эксперт АЦ
  3 - Сопровождение АЦ
  4 - Иные пользователи
';
COMMENT ON COLUMN public.price_analysis_user.ppz_type_id IS 'Направление пользователя';
COMMENT ON COLUMN public.price_analysis_user.user_id IS 'Идентификатор Эксперта АЦ';
COMMENT ON COLUMN public.price_analysis_user.env_type_id IS '
  Тип ландшафта:
  1 - Продуктивный
  2 - Тестовый
';
COMMENT ON COLUMN public.price_analysis_user.start_date IS 'Период действия с...';
COMMENT ON COLUMN public.price_analysis_user.end_date IS 'Период действия по...';
COMMENT ON COLUMN public.price_analysis_user.is_removed IS 'Признак удаления';
COMMENT ON COLUMN public.price_analysis_user.created_at IS 'Дата создания';
COMMENT ON COLUMN public.price_analysis_user.changed_at IS 'Дата изменения';
COMMENT ON COLUMN public.price_analysis_user.created_by IS 'Идентификатор создателя';
COMMENT ON COLUMN public.price_analysis_user.changed_by IS 'Идентификатор того кто изменил';
