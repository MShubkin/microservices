

CREATE TABLE public.plan_item (
    uuid uuid NOT NULL,
    id bigint NOT NULL,
    plan_uuid uuid NOT NULL,
    description_internal text,
    currency_id smallint NOT NULL,
    currency_rate bigint NOT NULL,
    category_id smallint NOT NULL,
    product_type_id smallint NOT NULL,
    budget_item_id smallint NOT NULL,
    okved2_id smallint NOT NULL,
    okato_id integer,
    unit_id smallint NOT NULL,
    payment_balance_item_id smallint NOT NULL,
    is_not_russian_delivery boolean NOT NULL,
    note text,
    quantity bigint NOT NULL,
    okpd2_id bigint DEFAULT 0 NOT NULL,
    delivery_basis character varying(1000) DEFAULT ''::text NOT NULL,
    price bigint DEFAULT 0 NOT NULL,
    currency_rate_date date,
    vat_id smallint DEFAULT 0 NOT NULL,
    transportation_price bigint,
    transportation_vat_id smallint DEFAULT 0 NOT NULL,
    transportation_sum_included_vat bigint,
    sum_excluded_vat bigint DEFAULT 0 NOT NULL,
    sum_vat bigint DEFAULT 0 NOT NULL,
    sum_included_vat bigint DEFAULT 0 NOT NULL,
    sum_excluded_vat_rub bigint DEFAULT 0 NOT NULL,
    sum_included_vat_rub bigint DEFAULT 0 NOT NULL,
    delivery_start_date date DEFAULT now() NOT NULL,
    delivery_end_date date DEFAULT now() NOT NULL,
    price_source_1_text character varying(1000),
    price_source_1_price bigint,
    price_source_1_date date,
    price_source_1_sum_included_vat bigint,
    price_source_2_text character varying(1000),
    price_source_2_price bigint,
    price_source_2_date date,
    price_source_2_sum_included_vat bigint,
    price_source_3_text character varying(1000),
    price_source_3_price bigint,
    price_source_3_date date,
    price_source_3_sum_included_vat bigint,
    is_analog_allowed boolean,
    analog_price bigint,
    analog_text character varying(1000),
    analog_producer_id integer,
    analog_country_id smallint,
    analog_requirements character varying(1000),
    mark character varying(1000),
    mark_main character varying(1000),
    technical_characteristics character varying(1000),
    technical_requirements character varying(1000),
    gosts character varying(1000),
    material_code_ius_local character varying(40),
    material_code_ius_mtr character varying(18),
    is_serial boolean,
    pzp_code character varying(40),
    nomenclature_group_id smallint,
    source_country_id smallint,
    producer_country_id smallint,
    producer_id integer,
    previous_price bigint,
    previous_delivery_date date,
    investment_project_id integer,
    investment_project_code character varying(63),
    is_dealer boolean,
    is_material_registry boolean,
    certificate_holder_id integer,
    certificate_text character varying(1000),
    certificate_number character varying(25),
    is_centralized_delivery boolean,
    centralized_sum bigint,
    prepayment_percent bigint,
    payment_delay smallint,
    psd_price bigint,
    psd_date date,
    psd_code character varying(1000),
    onm_price bigint,
    material_registry_price bigint,
    expert_price bigint,
    expert_sum_included_vat bigint,
    is_removed boolean DEFAULT false NOT NULL,
    created_at_date date DEFAULT now() NOT NULL,
    changed_at_date date,
    repair_inventory_number character varying(100),
    repair_summary_code character varying(60),
    repair_text character varying(1000),
    repair_plan_code character varying(100),
    plan_id_lotting character varying,
    uuid_item_proposal uuid,
    pricing_quantity bigint,
    pricing_unit_id smallint,
    pricing_price bigint,
    pricing_price_rub bigint,
    pricing_vat_id smallint DEFAULT 0 NOT NULL,
    pricing_currency_id smallint,
    pricing_currency_rate bigint,
    pricing_currency_rate_date date,
    pricing_sum_excluded_vat bigint,
    pricing_sum_excluded_vat_rub bigint,
    pricing_sum_included_vat bigint,
    pricing_sum_included_vat_rub bigint,
    pricing_sum_vat bigint,
    pricing_sum_vat_rub bigint,
    pricing_transportation_vat_id smallint DEFAULT 0 NOT NULL,
    pricing_transportation_price bigint DEFAULT 0,
    pricing_transportation_price_rub bigint DEFAULT 0,
    pricing_transportation_sum_vat bigint DEFAULT 0,
    pricing_transportation_sum_vat_rub bigint DEFAULT 0,
    pricing_transportation_sum_included_vat bigint DEFAULT 0,
    pricing_transportation_sum_included_vat_rub bigint DEFAULT 0,
    pricing_total_sum bigint,
    pricing_total_sum_rub bigint,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    created_by integer NOT NULL,
    changed_by integer NOT NULL,
    pricing_created_at timestamp without time zone DEFAULT '2024-12-23 00:00:00'::timestamp without time zone NOT NULL,
    pricing_changed_at timestamp without time zone DEFAULT '2024-12-23 00:00:00'::timestamp without time zone NOT NULL,
    sum_vat_rub bigint DEFAULT 0 NOT NULL,
    number smallint DEFAULT 0 NOT NULL
);

ALTER TABLE ONLY public.plan_item
    ADD CONSTRAINT plan_item_pkey PRIMARY KEY (uuid);

COMMENT ON COLUMN public.plan_item.uuid IS 'Уникальный идентификатор записи';


COMMENT ON COLUMN public.plan_item.id IS 'Внешней идентификатор записи (10 значное число)';



COMMENT ON COLUMN public.plan_item.plan_uuid IS 'Уникальный идентификатор заголовка ППЗ к которому принадлежит запись';



COMMENT ON COLUMN public.plan_item.created_at IS 'Дата создания';



COMMENT ON COLUMN public.plan_item.changed_at IS 'Дата изменения';



COMMENT ON COLUMN public.plan_item.created_by IS 'Идентификатор создателя';



COMMENT ON COLUMN public.plan_item.changed_by IS 'Идентификатор того кто изменил';



