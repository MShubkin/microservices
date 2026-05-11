-- TODO: Понять за что отвечает эта таблица и переписать.


CREATE TABLE public.department(
    uuid uuid NOT NULL PRIMARY KEY UNIQUE,
    department_id SMALLINT NOT NULL,
    pricing_department_expert_id BIGINT,
    -- How is this different from decision status.
    decision_process SMALLINT NOT NULL,
    decision_status SMALLINT NOT NULL,
    scheduled_decision_date date,
    -- TODO: Найти название по лучше.
    pricing_department_notification_date date,
    -- Assuming we use user ids here.
    pricing_department_notification_recipients INTEGER[] NOT NULL,
    -- TODO: Найти название по лучше.
    unit_notification_date date,
    -- Assuming we use user ids here.
    unit_notification_recipients INTEGER[] NOT NULL,
    department_assigned_date date NOT NULL,
    unit_assigned_date date NOT NULL,
    expert_assigned_date date NOT NULL,
    decision_date date NOT NULL,
    decision VARCHAR(255),
    decision_comment VARCHAR(2047),
    pricing_department_expert_comment VARCHAR(2047),
    -- Assumed to be a monetary unit.
    total_when_decision BIGINT,
    delayed_status_switch BOOLEAN NOT NULL,
    decision_outside_asez BOOLEAN NOT NULL,
    automatic_designation BOOLEAN NOT NULL,
    -- Should probably be united with `automatic_designation`.
    -- Probably joins `object_route.uuid`.
    automatic_assignment_route_uuid uuid,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    -- TODO: We should probably be inserting some kind of id here
    --       Then we can return to varchar(10)
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
)
TABLESPACE pg_default;

COMMENT ON TABLE public.department
    IS 'Справочник «Департамент (организация) АЦ»';

COMMENT ON COLUMN public.department.created_by
    IS 'Идентификатор создателя';

COMMENT ON COLUMN public.department.changed_by
    IS 'Идентификатор того кто изменил';

COMMENT ON COLUMN public.department.created_at
    IS 'Дата создания';

COMMENT ON COLUMN public.department.changed_at
    IS 'Дата изменения';
