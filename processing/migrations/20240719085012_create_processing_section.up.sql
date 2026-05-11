CREATE TYPE entity_type AS ENUM ('plan', 'contract_amendment', 'agenda', 'agenda_item', 'protocol', 'protocol_item');
CREATE TYPE extra_fields AS (
    entity entity_type,
    fields VARCHAR[]
);

CREATE TABLE public.processing_section (
    section_id smallint NOT NULL,
    base_plan_filters jsonb[] NOT NULL,
    extra_plan_status_filters smallint[],
    other_filters jsonb[],
    user_filter_column character varying(63),
    year_offset smallint,
    has_agenda_item_filter boolean NOT NULL,
    has_protocol_item_filter boolean NOT NULL,
    protocol_type smallint,
    agenda_dependency_on_protocol boolean DEFAULT false,
    user_priority_filter_fields character varying(63)[],
    extra_fields public.extra_fields[]
);

ALTER TABLE ONLY public.processing_section
    ADD CONSTRAINT processing_section_pkey PRIMARY KEY (section_id);

COMMENT ON TABLE public.processing_section IS 'Таблица хранит секции';



COMMENT ON COLUMN public.processing_section.section_id IS 'Идентификатор секции';



COMMENT ON COLUMN public.processing_section.base_plan_filters IS 'Фильтры, которые будут применены к ППЗ/ДС в любой из выборок по секции';



COMMENT ON COLUMN public.processing_section.extra_plan_status_filters IS 'Фильтры, которые будут применены к ППЗ/ДС в зависимости от сложившихся условий в секции';



COMMENT ON COLUMN public.processing_section.other_filters IS 'Фильтры, которые будут применены к тем сущностям, которые были указаны в данных';



COMMENT ON COLUMN public.processing_section.user_filter_column IS 'Поле ППЗ/ДС, к которому будет применен фильтр с входящим `user_id`';



COMMENT ON COLUMN public.processing_section.year_offset IS 'Фильтр по `year` c учетом указанного смещения';



COMMENT ON COLUMN public.processing_section.has_agenda_item_filter IS 'Проверка на существование смежных agenda_item для ППЗ/ДС';



COMMENT ON COLUMN public.processing_section.has_protocol_item_filter IS 'Проверка на существование смежных protocol_item для ППЗ/ДС';



COMMENT ON COLUMN public.processing_section.protocol_type IS 'Тип Протокола у существующего элемента Протокола';



COMMENT ON COLUMN public.processing_section.agenda_dependency_on_protocol IS 'Выборка по Повестке зависит от Протокола таким образом, что если у элемента Протокола result_id=3,
    то данные по Повестке и Протоколу не будут выбраны';



COMMENT ON COLUMN public.processing_section.user_priority_filter_fields IS 'Фильтры для полей по секциям, которые не будут применены, если от пользователя
     они пришли в ручном режиме';

INSERT INTO public.processing_section VALUES 
    (1, '{"{\"column_id\": \"commission_kind_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [1]}]}","{\"column_id\": \"commission_date\", \"value_list\": [{\"operator\": \"not_equals\", \"filter_values\": [null]}]}"}', '{221,222,223,225,341,342,343,345,351,352,353,355,251}', '{"{\"entity\": \"plan\", \"filters\": [{\"column_id\": \"purchasing_type_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [2]}]}]}"}', NULL, NULL, true, true, 1, true, '{status_id}', '{"(agenda,\"{agenda_id,agenda_status_id}\")","(protocol,\"{protocol_date,registration_number,protocol_id,protocol_status_id}\")","(protocol_item,\"{commission_sum_excluded_vat,commission_percent_economy,commission_economy_sum_excluded_vat}\")"}'),
    (2, '{"{\"column_id\": \"status_id\", \"value_list\": [{\"operator\": \"in\", \"filter_values\": [221, 222, 223, 225, 341, 342, 343, 345, 351, 352, 353, 355, 251]}]}","{\"column_id\": \"commission_kind_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [1]}]}","{\"column_id\": \"purchasing_type_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [2]}]}"}', NULL, NULL, NULL, NULL, false, false, NULL, false, NULL, NULL),
    (3, '{"{\"column_id\": \"commission_kind_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [2]}]}"}', '{252}', '{"{\"entity\": \"plan\", \"filters\": [{\"column_id\": \"purchasing_type_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [2]}]}]}"}', NULL, NULL, false, true, 2, false, '{status_id}', '{"(protocol,\"{protocol_date,registration_number,protocol_id,protocol_status_id}\")","(protocol_item,\"{commission_sum_excluded_vat,commission_percent_economy,commission_economy_sum_excluded_vat}\")"}'),
    (6, '{"{\"column_id\": \"status_id\", \"value_list\": [{\"operator\": \"in\", \"filter_values\": [221, 341, 351]}]}"}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (8, '{"{\"column_id\": \"status_id\", \"value_list\": [{\"operator\": \"in\", \"filter_values\": [223, 343, 353]}]}"}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (9, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (7, '{"{\"column_id\": \"status_id\", \"value_list\": [{\"operator\": \"in\", \"filter_values\": [222, 342, 352]}]}","{\"column_id\": \"is_check_documentation\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [true]}]}"}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (10, '{"{\"column_id\": \"status_id\", \"value_list\": [{\"operator\": \"in\", \"filter_values\": [222, 342, 352]}]}","{\"column_id\": \"is_check_documentation\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [false]}]}"}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (11, '{"{\"column_id\": \"pricing_expert_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [null]}]}"}', NULL, NULL, NULL, NULL, false, false, NULL, false, NULL, NULL),
    (12, '{}', NULL, NULL, 'pricing_expert_id', NULL, false, false, NULL, false, NULL, NULL),
    (13, '{}', NULL, NULL, NULL, NULL, false, false, NULL, false, NULL, NULL),
    (14, '{}', NULL, NULL, NULL, NULL, false, false, NULL, false, NULL, NULL),
    (15, '{}', NULL, NULL, NULL, NULL, false, false, NULL, false, NULL, NULL),
    (0, '{}', NULL, NULL, NULL, NULL, false, false, NULL, false, NULL, NULL),
    (5, '{}', NULL, '{"{\"entity\": \"plan\", \"filters\": [{\"column_id\": \"purchasing_type_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [2]}]}]}"}', NULL, NULL, true, true, NULL, true, NULL, '{"(agenda,\"{agenda_id,agenda_status_id}\")","(protocol,\"{protocol_date,registration_number,protocol_id,protocol_status_id}\")","(protocol_item,\"{commission_sum_excluded_vat,commission_percent_economy,commission_economy_sum_excluded_vat}\")"}'),
    (4, '{"{\"column_id\": \"status_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [253]}]}","{\"column_id\": \"commission_kind_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [3]}]}"}', NULL, '{"{\"entity\": \"plan\", \"filters\": [{\"column_id\": \"purchasing_type_id\", \"value_list\": [{\"operator\": \"equals\", \"filter_values\": [2]}]}]}"}', NULL, NULL, false, false, NULL, false, NULL, NULL),
    (16, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (17, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (18, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (19, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (20, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (21, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (22, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (23, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (24, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (25, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (26, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (27, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (28, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL),
    (29, '{}', NULL, NULL, NULL, 0, false, false, NULL, false, NULL, NULL);





