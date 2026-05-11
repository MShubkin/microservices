-- Table: public.plan_item


CREATE TABLE public.plan_items_legacy
(
  plan_uuid uuid NOT NULL,
  uuid uuid NOT NULL PRIMARY KEY,
  "number" SMALLINT NOT NULL DEFAULT 0,
  description_internal character varying(1000) NOT NULL DEFAULT ''::character varying,
  description_external character varying(1000) NOT NULL DEFAULT ''::character varying,
  product_type_id SMALLINT NOT NULL DEFAULT 0,
  category_id SMALLINT NOT NULL DEFAULT 0,
  budget_item_id SMALLINT NOT NULL DEFAULT 0,
  payment_balance_item_id SMALLINT NOT NULL DEFAULT 0,
  consumer_id INTEGER NOT NULL DEFAULT 0,
  okpd2_id INTEGER NOT NULL DEFAULT 0,
  okved2_id SMALLINT NOT NULL DEFAULT 0,
  okato_id INTEGER NOT NULL DEFAULT 0,
  is_not_russian_delivery BOOLEAN NOT NULL DEFAULT false,
  delivery_basis character varying(1000) NOT NULL DEFAULT ''::character varying,
  unit_id SMALLINT NOT NULL DEFAULT 0,
  quantity BIGINT NOT NULL DEFAULT 0,
  price BIGINT NOT NULL DEFAULT 0,
  price_unit INTEGER NOT NULL DEFAULT 0,
  currency_id SMALLINT NOT NULL DEFAULT 0,
  currency_rate BIGINT NOT NULL DEFAULT 0,
  currency_rate_date date NOT NULL DEFAULT '1900-01-01'::date,
  vat_id SMALLINT NOT NULL DEFAULT 0,
  transportation_price BIGINT NOT NULL DEFAULT 0,
  transportation_vat_id SMALLINT NOT NULL DEFAULT 0,
  transportation_sum_included_vat BIGINT NOT NULL DEFAULT 0,
  sum_excluded_vat BIGINT NOT NULL DEFAULT 0,
  sum_vat BIGINT NOT NULL DEFAULT 0,
  sum_included_vat BIGINT NOT NULL DEFAULT 0,
  sum_excluded_vat_rub BIGINT NOT NULL DEFAULT 0,
  sum_vat_rub BIGINT NOT NULL DEFAULT 0,
  sum_included_vat_rub BIGINT NOT NULL DEFAULT 0,
  delivery_start_date date NOT NULL DEFAULT '1900-01-01'::date,
  delivery_end_date date NOT NULL DEFAULT '1900-01-01'::date,
  price_source_1_text character varying(1000) NOT NULL DEFAULT ''::character varying,
  price_source_1_price BIGINT NOT NULL DEFAULT 0,
  price_source_1_date date NOT NULL DEFAULT '1900-01-01'::date,
  price_source_2_text character varying(1000) NOT NULL DEFAULT ''::character varying,
  price_source_2_price BIGINT NOT NULL DEFAULT 0,
  price_source_2_date date NOT NULL DEFAULT '1900-01-01'::date,
  price_source_3_text character varying(1000) NOT NULL DEFAULT ''::character varying,
  price_source_3_price BIGINT NOT NULL DEFAULT 0,
  price_source_3_date date NOT NULL DEFAULT '1900-01-01'::date,
  is_analog_allowed BOOLEAN NOT NULL DEFAULT false,
  analog_price BIGINT NOT NULL DEFAULT 0,
  analog_text character varying(1000) NOT NULL DEFAULT ''::character varying,
  analog_producer_id INTEGER NOT NULL DEFAULT 0,
  analog_producer_text character varying(1000) NOT NULL DEFAULT ''::character varying,
  analog_country_id SMALLINT NOT NULL DEFAULT 0,
  analog_requirements character varying(1000) NOT NULL DEFAULT ''::character varying,
  mark character varying(1000) NOT NULL DEFAULT ''::character varying,
  mark_main character varying(1000) NOT NULL DEFAULT ''::character varying,
  technical_characteristics character varying(1000) NOT NULL DEFAULT ''::character varying,
  technical_requirements character varying(1000) NOT NULL DEFAULT ''::character varying,
  gosts character varying(1000) NOT NULL DEFAULT ''::character varying,
  material_code_local character varying(40) NOT NULL DEFAULT ''::character varying,
  material_code_ius_mtr character varying(18) NOT NULL DEFAULT ''::character varying,
  is_serial BOOLEAN NOT NULL DEFAULT false,
  pzp_code character varying(40) NOT NULL DEFAULT ''::character varying,
  nomenclature_group_id SMALLINT NOT NULL DEFAULT 0,
  source_country_id SMALLINT NOT NULL DEFAULT 0,
  producer_country_id SMALLINT NOT NULL DEFAULT 0,
  producer_id INTEGER NOT NULL DEFAULT 0,
  previous_price BIGINT NOT NULL DEFAULT 0,
  previous_delivery_date date NOT NULL DEFAULT '1900-01-01'::date,
  investment_project_id INTEGER NOT NULL DEFAULT 0,
  is_dealer BOOLEAN NOT NULL DEFAULT false,
  is_material_registry BOOLEAN NOT NULL DEFAULT false,
  certificate_holder_id INTEGER NOT NULL DEFAULT 0,
  certificate_text character varying(1000) NOT NULL DEFAULT ''::character varying,
  certificate_number character varying(25) NOT NULL DEFAULT ''::character varying,
  is_centralized_delivery BOOLEAN NOT NULL DEFAULT false,
  centralized_sum BIGINT NOT NULL DEFAULT 0,
  prepayment_percent BIGINT NOT NULL DEFAULT 0,
  payment_delay SMALLINT NOT NULL DEFAULT 0,
  psd_price BIGINT NOT NULL DEFAULT 0,
  psd_date date NOT NULL DEFAULT '1900-01-01'::date,
  psd_code character varying(1000) NOT NULL DEFAULT ''::character varying,
  onm_price BIGINT NOT NULL DEFAULT 0,
  material_registry_price BIGINT NOT NULL DEFAULT 0,
  expert_price BIGINT NOT NULL DEFAULT 0,
  expert_sum_included_vat BIGINT NOT NULL DEFAULT 0,
  pricing_quantity BIGINT NOT NULL DEFAULT 0,
  pricing_price BIGINT NOT NULL DEFAULT 0,
  pricing_vat_id SMALLINT NOT NULL DEFAULT 0,
  pricing_currency_id SMALLINT NOT NULL DEFAULT 0,
  pricing_currency_rate BIGINT NOT NULL DEFAULT 0,
  pricing_transportation_price BIGINT NOT NULL DEFAULT 0,
  pricing_transportation_vat_id SMALLINT NOT NULL DEFAULT 0,
  pricing_unit_id SMALLINT NOT NULL DEFAULT 0,
  pricing_department_id INTEGER NOT NULL DEFAULT 0,
  pricing_expert_id INTEGER NOT NULL DEFAULT 0,
  pricing_method_id SMALLINT NOT NULL DEFAULT 0,
  pricing_resume character varying(1000) NOT NULL DEFAULT ''::character varying,
  status_id SMALLINT NOT NULL DEFAULT 0,
  is_removed BOOLEAN NOT NULL DEFAULT false,
  created_at timestamp without time zone NOT NULL DEFAULT '1970-01-01 00:00:00'::timestamp without time zone,
  created_by INTEGER NOT NULL DEFAULT 0,
  changed_at timestamp without time zone NOT NULL DEFAULT '1970-01-01 00:00:00'::timestamp without time zone,
  changed_by INTEGER NOT NULL DEFAULT 0,
  active_uuid uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::uuid,
  items_number SMALLINT NOT NULL DEFAULT 0,
  repair_inventory_number character varying(100) NOT NULL DEFAULT ''::character varying,
  repair_summary_code character varying(60) NOT NULL DEFAULT ''::character varying,
  repair_approved_at timestamp without time zone NOT NULL DEFAULT '1900-01-01 00:00:00'::timestamp without time zone,
  repair_text character varying(1000) NOT NULL DEFAULT ''::character varying,
  repair_plan_code character varying(100) NOT NULL DEFAULT ''::character varying,
  budget_uuid uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::uuid,
  is_lot BOOLEAN NOT NULL DEFAULT false,
  lot_uuid uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::uuid,
  rus_pp_2013_mark BOOLEAN DEFAULT false,
  okpd2_pp_2013_id BIGINT DEFAULT 0,
  rpp_rf_pp_719_id BIGINT DEFAULT 0,
  rpp_eaes_pp_616_id BIGINT DEFAULT 0,
  errrp_pp_878_id BIGINT DEFAULT 0,
  rf_products_reason_id SMALLINT DEFAULT 0,
  not_exist_in_gisp_reestr BOOLEAN DEFAULT false,
  rpp_rf_pp_719_number character varying(20) NOT NULL DEFAULT ''::character varying,
  rpp_eaes_pp_616_number character varying(20) NOT NULL DEFAULT ''::character varying,
  errrp_pp_878_number character varying(20) NOT NULL DEFAULT ''::character varying
)
WITH (
  OIDS=FALSE
);
-- ALTER TABLE public.plan_item
--   OWNER TO srm;
-- GRANT ALL ON TABLE public.plan_item TO srm;

-- Index: public.plan_item_plan_uuid_uuid_idx

-- DROP INDEX public.plan_item_plan_uuid_uuid_idx;

CREATE INDEX IF NOT EXISTS legacy_plan_item_plan_uuid_uuid_idx
  ON public.plan_items_legacy
  USING btree
  (plan_uuid, uuid);

-- Index: public.plan_item_uuid_idx

-- DROP INDEX public.plan_item_uuid_idx;

CREATE INDEX IF NOT EXISTS legacy_plan_item_uuid_idx
  ON public.plan_items_legacy
  USING btree
  (uuid);

