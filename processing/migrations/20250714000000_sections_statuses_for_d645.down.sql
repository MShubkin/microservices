
-- Statuses for estimated commission in person (removed 371, 372, 373, 375)
UPDATE processing_section SET extra_plan_status_filters = '{221,222,223,225,341,342,343,345,351,352,353,355,251}' WHERE section_id = 1;


-- Statuses for all plans for estimated commission (removed 371, 372, 373, 375)
UPDATE processing_section SET base_plan_filters = ARRAY['{"column_id": "status_id", "value_list": [{"operator": "in", "filter_values": [221, 222, 223, 225, 341, 342, 343, 345, 351, 352, 353, 355, 251]}]}'::jsonb, '{"column_id": "commission_kind_id", "value_list":[{"operator": "equals", "filter_values": [1]}]}'::jsonb, '{"column_id": "purchasing_type_id", "value_list": [{"operator": "equals", "filter_values": [2]}]}'::jsonb] WHERE section_id = 2;

-- Statuses for pricing assign expert (removed 371)
UPDATE processing_section SET base_plan_filters = ARRAY['{"column_id": "status_id", "value_list": [{"operator": "in", "filter_values": [221, 341, 351]}]}'::jsonb] WHERE section_id = 6;

-- Statuses for pricing determine price (removed 372)
UPDATE processing_section SET base_plan_filters = ARRAY['{"column_id": "status_id", "value_list": [{"operator": "in", "filter_values": [222, 342, 352]}]}'::jsonb, '{"column_id": "is_check_documentation", "value_list": [{"operator": "equals", "filter_values": [true]}]}'::jsonb] WHERE section_id = 7;

-- Statuses for pricing approve price (removed 373)
UPDATE processing_section SET base_plan_filters = ARRAY['{"column_id": "status_id", "value_list": [{"operator": "in", "filter_values": [223, 343, 353]}]}'::jsonb] WHERE section_id = 8;

-- Statuses for pricing primary control (removed 372)
UPDATE processing_section SET base_plan_filters = ARRAY['{"column_id": "status_id", "value_list": [{"operator": "in", "filter_values": [222, 342, 352]}]}'::jsonb, '{"column_id": "is_check_documentation", "value_list": [{"operator": "equals", "filter_values": [false]}]}'::jsonb] WHERE section_id = 10;
