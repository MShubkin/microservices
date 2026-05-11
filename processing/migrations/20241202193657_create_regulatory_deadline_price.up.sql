

CREATE TABLE public.regulatory_deadline_price (
    uuid UUID NOT NULL PRIMARY KEY,
    section INTEGER NOT NULL,
    field_id INTEGER,
    color_scheme_id INTEGER NOT NULL,
    type_criticality INTEGER NOT NULL,
    start_day INTEGER NOT NULL,
    end_day INTEGER NOT NULL,
    created_by INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    changed_by INTEGER NOT NULL,
    changed_at TIMESTAMP NOT NULL,
    status BOOLEAN
) TABLESPACE pg_default;
