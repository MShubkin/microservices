-- Table: public.organization
CREATE TABLE public.organization
(
    uuid uuid NOT NULL,
    id INTEGER DEFAULT 0,
    nsi_code INTEGER DEFAULT 0,
    inn VARCHAR(12) NOT NULL DEFAULT '',
    kpp CHARACTER(10) NOT NULL DEFAULT '',
    text VARCHAR(255) NOT NULL DEFAULT '',
    text_full VARCHAR(255) NOT NULL DEFAULT '',
    source VARCHAR(20) NOT NULL DEFAULT '',
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    form_id SMALLINT DEFAULT 0,
    ogrn VARCHAR(15) NOT NULL DEFAULT '',
    etp_code INTEGER DEFAULT 0,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER DEFAULT 0,
    changed_by INTEGER DEFAULT 0,
    CONSTRAINT organization_pkey PRIMARY KEY (uuid)
);
