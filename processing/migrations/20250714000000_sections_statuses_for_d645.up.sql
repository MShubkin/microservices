
-- Statuses for estimated commission in person (added 371, 372, 373, 375)
UPDATE processing_section SET extra_plan_status_filters = '{221,222,223,225,341,342,343,345,351,352,353,355,371,372,373,375,251}' WHERE section_id = 1;


-- Statuses for all plans for estimated commission (added 371, 372, 373, 375)
UPDATE processing_section SET base_plan_filters = ARRAY['{"column_id": "status_id", "value_list": [{"operator": "in", "filter_values": [221, 222, 223, 225, 341, 342, 343, 345, 351, 352, 353, 355, 371, 372, 373, 375, 251]}]}'::jsonb, '{"column_id": "commission_kind_id", "value_list":[{"operator": "equals", "filter_values": [1]}]}'::jsonb, '{"column_id": "purchasing_type_id", "value_list": [{"operator": "equals", "filter_values": [2]}]}'::jsonb] WHERE section_id = 2;

-- Statuses for pricing assign expert (added 371)
UPDATE processing_section SET base_plan_filters = ARRAY['{"column_id": "status_id", "value_list": [{"operator": "in", "filter_values": [221, 341, 351, 371]}]}'::jsonb] WHERE section_id = 6;

-- Statuses for pricing determine price (added 372)
UPDATE processing_section SET base_plan_filters = ARRAY['{"column_id": "status_id", "value_list": [{"operator": "in", "filter_values": [222, 342, 352, 372]}]}'::jsonb, '{"column_id": "is_check_documentation", "value_list": [{"operator": "equals", "filter_values": [true]}]}'::jsonb] WHERE section_id = 7;

-- Statuses for pricing approve price (added 373)
UPDATE processing_section SET base_plan_filters = ARRAY['{"column_id": "status_id", "value_list": [{"operator": "in", "filter_values": [223, 343, 353, 373]}]}'::jsonb] WHERE section_id = 8;

-- Statuses for pricing primary control (added 372)
UPDATE processing_section SET base_plan_filters = ARRAY['{"column_id": "status_id", "value_list": [{"operator": "in", "filter_values": [222, 342, 352, 372]}]}'::jsonb, '{"column_id": "is_check_documentation", "value_list": [{"operator": "equals", "filter_values": [false]}]}'::jsonb] WHERE section_id = 10;
