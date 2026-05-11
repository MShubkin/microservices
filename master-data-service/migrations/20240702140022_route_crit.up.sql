CREATE TABLE public.route_crit (
    field_name character varying NOT NULL,
    predicate jsonb NOT NULL,
    is_removed boolean DEFAULT false NOT NULL,
    created_at timestamp without time zone NOT NULL,
    created_by integer NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    changed_by integer NOT NULL,
    route_uuid uuid NOT NULL
);
