

CREATE TABLE public.contract_amendment (
    uuid uuid NOT NULL,
    id bigint NOT NULL,
    version_type smallint DEFAULT 0 NOT NULL,
    version_number smallint DEFAULT 0 NOT NULL,
    active_uuid uuid DEFAULT '00000000-0000-0000-0000-000000000000'::uuid NOT NULL,
    source_uuid uuid DEFAULT '00000000-0000-0000-0000-000000000000'::uuid NOT NULL,
    is_pur_asbu boolean DEFAULT false NOT NULL,
    system_number character varying(18) DEFAULT ''::character varying NOT NULL,
    external_number character varying(60) DEFAULT ''::character varying NOT NULL,
    branch character varying(200) DEFAULT ''::character varying NOT NULL,
    customer_id integer NOT NULL,
    declarant_id integer DEFAULT 0 NOT NULL,
    agent_id integer DEFAULT 0 NOT NULL,
    assignee_id integer DEFAULT 0 NOT NULL,
    project_institute_id integer DEFAULT 0 NOT NULL,
    organizer_id integer DEFAULT 0 NOT NULL,
    initiator_user_id integer DEFAULT 0 NOT NULL,
    tender_user_id integer DEFAULT 0 NOT NULL,
    year smallint DEFAULT (date_part('year'::text, now()))::smallint NOT NULL,
    purchasing_type_id smallint NOT NULL,
    purchasing_method_id smallint DEFAULT 0 NOT NULL,
    section_id smallint NOT NULL,
    funding_source_id smallint DEFAULT 0 NOT NULL,
    single_supplier_reason_id smallint DEFAULT 0 NOT NULL,
    number_cgg character varying(64) DEFAULT ''::character varying NOT NULL,
    contract_system_number character varying(18) DEFAULT ''::character varying NOT NULL,
    contract_external_number character varying(60) DEFAULT ''::character varying NOT NULL,
    number_eis character varying(23) DEFAULT ''::character varying NOT NULL,
    supplier_id integer NOT NULL,
    contract_subject character varying(2000) DEFAULT ''::character varying NOT NULL,
    contract_type_id smallint DEFAULT 0 NOT NULL,
    accepted_volume_included_vat_rub bigint DEFAULT 0 NOT NULL,
    is_banking_support boolean DEFAULT false NOT NULL,
    is_with_amendments boolean DEFAULT false NOT NULL,
    is_secret_state boolean DEFAULT false NOT NULL,
    is_secret_commercial boolean DEFAULT false NOT NULL,
    rationale character varying(5000) DEFAULT ''::character varying NOT NULL,
    funding_availability character varying(5000) DEFAULT ''::character varying NOT NULL,
    is_chairman_order boolean DEFAULT false NOT NULL,
    is_chairman_order_secret boolean DEFAULT false NOT NULL,
    chairman_order_number character varying(50) DEFAULT ''::character varying NOT NULL,
    chairman_order_date date DEFAULT '1900-01-01'::date NOT NULL,
    is_vice_chairman_order boolean DEFAULT false NOT NULL,
    is_with_approval boolean DEFAULT false NOT NULL,
    is_need_for_departments boolean DEFAULT false NOT NULL,
    is_sum_increase_was_specified boolean DEFAULT false NOT NULL,
    is_sum_changed_via_key_rate boolean DEFAULT false NOT NULL,
    is_material_registry boolean DEFAULT false NOT NULL,
    is_to_publish boolean DEFAULT false NOT NULL,
    repair_stage_id smallint DEFAULT 0 NOT NULL,
    vat_id smallint DEFAULT 0 NOT NULL,
    sum_excluded_vat bigint NOT NULL,
    sum_vat bigint DEFAULT 0 NOT NULL,
    sum_included_vat bigint DEFAULT 0 NOT NULL,
    currency_id smallint NOT NULL,
    currency_rate bigint DEFAULT 0 NOT NULL,
    sum_excluded_vat_rub bigint DEFAULT 0 NOT NULL,
    sum_vat_rub bigint DEFAULT 0 NOT NULL,
    sum_included_vat_rub bigint DEFAULT 0 NOT NULL,
    initial_vat_id smallint DEFAULT 0 NOT NULL,
    initial_sum_excluded_vat bigint DEFAULT 0 NOT NULL,
    initial_sum_vat bigint DEFAULT 0 NOT NULL,
    initial_sum_included_vat bigint DEFAULT 0 NOT NULL,
    initial_currency_id smallint DEFAULT 0 NOT NULL,
    initial_currency_rate bigint DEFAULT 0 NOT NULL,
    initial_sum_excluded_vat_rub bigint DEFAULT 0 NOT NULL,
    initial_sum_vat_rub bigint DEFAULT 0 NOT NULL,
    initial_sum_included_vat_rub bigint DEFAULT 0 NOT NULL,
    previous_vat_id smallint DEFAULT 0 NOT NULL,
    previous_sum_excluded_vat bigint DEFAULT 0 NOT NULL,
    previous_sum_vat bigint DEFAULT 0 NOT NULL,
    previous_sum_included_vat bigint DEFAULT 0 NOT NULL,
    previous_currency_id smallint DEFAULT 0 NOT NULL,
    previous_currency_rate bigint DEFAULT 0 NOT NULL,
    previous_sum_excluded_vat_rub bigint DEFAULT 0 NOT NULL,
    previous_sum_vat_rub bigint DEFAULT 0 NOT NULL,
    previous_sum_included_vat_rub bigint DEFAULT 0 NOT NULL,
    delta_sum_excluded_vat bigint DEFAULT 0 NOT NULL,
    delta_sum_vat bigint DEFAULT 0 NOT NULL,
    delta_sum_included_vat bigint DEFAULT 0 NOT NULL,
    delta_sum_excluded_vat_rub bigint DEFAULT 0 NOT NULL,
    delta_sum_vat_rub bigint DEFAULT 0 NOT NULL,
    delta_sum_included_vat_rub bigint DEFAULT 0 NOT NULL,
    sign_date date DEFAULT '1900-01-01'::date NOT NULL,
    close_date date DEFAULT '1900-01-01'::date NOT NULL,
    termination_date date DEFAULT '1900-01-01'::date NOT NULL,
    start_date date DEFAULT '1900-01-01'::date NOT NULL,
    end_date date DEFAULT '1900-01-01'::date NOT NULL,
    whole_start_date date DEFAULT '1900-01-01'::date NOT NULL,
    whole_end_date date DEFAULT '1900-01-01'::date NOT NULL,
    initial_start_date date DEFAULT '1900-01-01'::date NOT NULL,
    initial_end_date date DEFAULT '1900-01-01'::date NOT NULL,
    initial_whole_start_date date DEFAULT '1900-01-01'::date NOT NULL,
    initial_whole_end_date date DEFAULT '1900-01-01'::date NOT NULL,
    previous_start_date date DEFAULT '1900-01-01'::date NOT NULL,
    previous_end_date date DEFAULT '1900-01-01'::date NOT NULL,
    previous_whole_start_date date DEFAULT '1900-01-01'::date NOT NULL,
    previous_whole_end_date date DEFAULT '1900-01-01'::date NOT NULL,
    is_priority_project boolean DEFAULT false NOT NULL,
    priority_project_document character varying(100) DEFAULT ''::character varying NOT NULL,
    is_priority_introductory boolean DEFAULT false NOT NULL,
    priority_introductory_date date DEFAULT '1900-01-01'::date NOT NULL,
    priority_introductory_document character varying(150) DEFAULT ''::character varying NOT NULL,
    is_priority_repair boolean DEFAULT false NOT NULL,
    priority_repair_document character varying(100) DEFAULT ''::character varying NOT NULL,
    is_priority_ozp boolean DEFAULT false NOT NULL,
    priority_ozp_document character varying(100) DEFAULT ''::character varying NOT NULL,
    is_priority_income_contract boolean DEFAULT false NOT NULL,
    priority_income_contract_document character varying(100) DEFAULT ''::character varying NOT NULL,
    priority_income_contract_partner_id integer DEFAULT 0 NOT NULL,
    priority_income_contract_partner_text character varying(100) DEFAULT ''::character varying NOT NULL,
    is_priority_other boolean DEFAULT false NOT NULL,
    is_headquarters boolean DEFAULT false NOT NULL,
    status_scheme_id smallint DEFAULT 0 NOT NULL,
    status_id smallint NOT NULL,
    is_approved_by_d646 boolean DEFAULT false NOT NULL,
    commission_kind_id smallint NOT NULL,
    budget_item_id smallint DEFAULT 0 NOT NULL,
    payment_balance_item_id smallint DEFAULT 0 NOT NULL,
    product_type_id smallint DEFAULT 0 NOT NULL,
    items_number smallint DEFAULT 0 NOT NULL,
    associated_plan_id bigint DEFAULT 0 NOT NULL,
    purchase_id character varying(10) DEFAULT ''::character varying NOT NULL,
    purchase_number_eis character varying(10) DEFAULT ''::character varying NOT NULL,
    quotation_id character varying(10) DEFAULT ''::character varying NOT NULL,
    contract_id character varying(10) DEFAULT ''::character varying NOT NULL,
    claim_id bigint DEFAULT 0 NOT NULL,
    is_removed boolean DEFAULT false NOT NULL,
    posting_date date DEFAULT '1900-01-01'::date NOT NULL,
    is_priority_far_eastern boolean DEFAULT false NOT NULL,
    executor_method_id smallint DEFAULT 0 NOT NULL,
    number_customer character varying(40) DEFAULT ''::text NOT NULL,
    commission_date date,
    kod_st_buda character varying(10),
    okdp2 character varying(20),
    category_id character varying(20),
    code_type smallint,
    expert_conclusion_id smallint,
    is_check_documentation boolean DEFAULT false NOT NULL,
    check_documentation_date timestamp without time zone,
    is_actual boolean DEFAULT true NOT NULL,
    contract_amendment_types integer[] DEFAULT ARRAY[]::integer[] NOT NULL,
    savings_accounting_id smallint DEFAULT 0 NOT NULL,
    savings_sum_excluded_vat bigint,
    savings_sum_excluded_vat_rub bigint,
    savings_sum_included_vat bigint,
    savings_sum_included_vat_rub bigint,
    pricing_method_id smallint DEFAULT 0 NOT NULL,
    pricing_expert_id integer,
    pricing_organization_unit_id smallint DEFAULT 0 NOT NULL,
    pricing_resume character varying(1000),
    pricing_competitive_note_for_expert character varying(1000),
    is_pricing_by_d646 boolean DEFAULT false NOT NULL,
    is_pricing_by_d647 boolean DEFAULT false NOT NULL,
    is_pricing_by_complectation boolean DEFAULT false NOT NULL,
    pricing_vat_id smallint DEFAULT 0 NOT NULL,
    pricing_currency_id smallint,
    pricing_currency_rate bigint,
    pricing_sum_excluded_vat bigint DEFAULT 0 NOT NULL,
    pricing_sum_excluded_vat_rub bigint,
    pricing_sum_included_vat bigint,
    pricing_sum_included_vat_rub bigint,
    pricing_sum_vat bigint,
    pricing_sum_vat_rub bigint,
    pricing_transportation_vat_id smallint DEFAULT 0 NOT NULL,
    pricing_transportation_price bigint,
    pricing_transportation_price_rub bigint,
    pricing_transportation_sum_vat bigint,
    pricing_transportation_sum_vat_rub bigint,
    pricing_transportation_sum_included_vat bigint,
    pricing_transportation_sum_included_vat_rub bigint,
    pricing_total_sum bigint,
    pricing_total_sum_rub bigint,
    pricing_delta_currency_id bigint,
    pricing_delta_currency_rate bigint,
    pricing_delta_sum_excluded_vat bigint,
    pricing_delta_sum_excluded_vat_rub bigint,
    pricing_delta_sum_included_vat bigint,
    pricing_delta_sum_included_vat_rub bigint,
    pricing_delta_sum_vat bigint,
    pricing_delta_sum_vat_rub bigint,
    pricing_delta_total_sum bigint,
    pricing_delta_total_sum_rub bigint,
    pricing_delta_transportation_price bigint,
    pricing_delta_transportation_sum_included_vat bigint,
    pricing_delta_transportation_sum_included_vat_rub bigint,
    pricing_delta_transportation_sum_vat bigint,
    pricing_delta_transportation_sum_vat_rub bigint,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    created_by integer NOT NULL,
    changed_by integer NOT NULL,
    pricing_started_at timestamp without time zone DEFAULT '1901-01-01 00:00:00'::timestamp without time zone NOT NULL,
    pricing_created_at timestamp without time zone DEFAULT '2024-12-23 00:00:00'::timestamp without time zone NOT NULL,
    pricing_changed_at timestamp without time zone DEFAULT '2024-12-23 00:00:00'::timestamp without time zone NOT NULL
);


ALTER TABLE ONLY public.contract_amendment
    ADD CONSTRAINT contract_amendment_pkey PRIMARY KEY (uuid);


COMMENT ON TABLE public.contract_amendment IS 'Таблица заголовков ДС по модели АСЕЗ-2.0';



COMMENT ON COLUMN public.contract_amendment.uuid IS 'Уникальный идентификатор записи';



COMMENT ON COLUMN public.contract_amendment.id IS 'Внешней идентификатор записи (10 значное число)';



COMMENT ON COLUMN public.contract_amendment.commission_date IS 'Предполагаемая дата заседания';



COMMENT ON COLUMN public.contract_amendment.is_check_documentation IS 'Документация проверена Экспертом АЦ';



COMMENT ON COLUMN public.contract_amendment.check_documentation_date IS 'Дата проверки документации Экспертом АЦ';



COMMENT ON COLUMN public.contract_amendment.pricing_resume IS 'Заключение Эксперта АЦ';



COMMENT ON COLUMN public.contract_amendment.created_at IS 'Дата создания';



COMMENT ON COLUMN public.contract_amendment.changed_at IS 'Дата изменения';



COMMENT ON COLUMN public.contract_amendment.created_by IS 'Идентификатор создателя';



COMMENT ON COLUMN public.contract_amendment.changed_by IS 'Идентификатор того кто изменил';
