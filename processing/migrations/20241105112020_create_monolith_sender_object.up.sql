

CREATE TABLE public.monolith_sender_object (
    id bigint NOT NULL,
    messages jsonb NOT NULL,
    locked boolean DEFAULT false NOT NULL,
    last_error text,
    created_at timestamp without time zone DEFAULT now()
);

ALTER TABLE ONLY public.monolith_sender_object
    ADD CONSTRAINT monolith_sender_object_pkey PRIMARY KEY (id);

CREATE SEQUENCE public.monolith_sender_object_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER TABLE ONLY public.monolith_sender_object ALTER COLUMN id SET DEFAULT nextval('public.monolith_sender_object_id_seq'::regclass);

ALTER SEQUENCE public.monolith_sender_object_id_seq OWNED BY public.monolith_sender_object.id;


