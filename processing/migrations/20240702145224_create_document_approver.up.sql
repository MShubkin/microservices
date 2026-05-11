

CREATE TABLE public.document_approver (
    uuid uuid NOT NULL,
    number integer NOT NULL,
    document_uuid uuid NOT NULL,
    plan_id bigint NOT NULL,
    department_id integer NOT NULL,
    planned_date date NOT NULL,
    started_at timestamp without time zone,
    division_id integer,
    division_assigned_at timestamp without time zone,
    expert_id integer,
    responded_at timestamp without time zone,
    response_id smallint,
    response_note character varying(1024),
    total_when_decision bigint,
    status_appr smallint DEFAULT 1 NOT NULL,
    responsible_person_id integer,
    is_auto boolean DEFAULT false NOT NULL,
    send_date_1 timestamp without time zone,
    send_users_1 integer[] DEFAULT '{}'::integer[] NOT NULL,
    send_date_2 timestamp without time zone,
    send_users_2 integer[] DEFAULT '{}'::integer[] NOT NULL,
    is_preapproved boolean DEFAULT false NOT NULL,
    is_removed boolean DEFAULT false NOT NULL,
    created_at timestamp without time zone DEFAULT '1900-01-01 00:00:00'::timestamp without time zone NOT NULL,
    changed_at timestamp without time zone DEFAULT '1900-01-01 00:00:00'::timestamp without time zone NOT NULL,
    created_by integer DEFAULT 0 NOT NULL,
    changed_by integer DEFAULT 0 NOT NULL,
    is_actual boolean DEFAULT false NOT NULL,
    route_id bigint[] DEFAULT ARRAY[]::bigint[] NOT NULL
);

COMMENT ON TABLE public.document_approver IS 'Согласование ПД (документов ППЗ/ДС)';



COMMENT ON COLUMN public.document_approver.uuid IS 'гуид записи таблицы (первичный ключ)';



COMMENT ON COLUMN public.document_approver.number IS 'порядковый номер согласования (Этап рассмотрения)  в рамках уникальной комбинации двух полей (plan id + department_id)';



COMMENT ON COLUMN public.document_approver.document_uuid IS 'гуид ППЗ/ДС (активной версии)';



COMMENT ON COLUMN public.document_approver.plan_id IS 'id ППЗ/ДС (как альтернатива document_uuid, т.к. у ППЗ могут быть версии с разными uuid, а данные ПД общие для всех версий. Id ППЗ/ДС не меняется в версиях)';



COMMENT ON COLUMN public.document_approver.department_id IS 'id Профильного  департамента';



COMMENT ON COLUMN public.document_approver.planned_date IS 'Плановая дата согласования (Рассмотреть до)';



COMMENT ON COLUMN public.document_approver.started_at IS 'Дата+время установки статуса status_appr=1 (в работе)';



COMMENT ON COLUMN public.document_approver.division_id IS 'id Подразделения ПД (из  записи department-id)';



COMMENT ON COLUMN public.document_approver.division_assigned_at IS 'Дата+время назначения подразделения ПД';



COMMENT ON COLUMN public.document_approver.expert_id IS 'Эксперт ПД, id пользователя, назначенного в качестве Эксперта ПД';



COMMENT ON COLUMN public.document_approver.responded_at IS 'Дата+время сохранения решения';



COMMENT ON COLUMN public.document_approver.response_id IS 'id Решения Эксперта ПД';



COMMENT ON COLUMN public.document_approver.response_note IS 'Комментарий к Решению Эксперта ПД';



COMMENT ON COLUMN public.document_approver.total_when_decision IS 'Сумма при согласовании';



COMMENT ON COLUMN public.document_approver.status_appr IS 'Технический статус';



COMMENT ON COLUMN public.document_approver.responsible_person_id IS 'id пользователя, назначившего Эксперта ПД (изменил поле expert_id)';



COMMENT ON COLUMN public.document_approver.is_auto IS 'Индикатор автоназначения ПД';



COMMENT ON COLUMN public.document_approver.send_date_1 IS 'Дата+время уведомления руководителей ПД';



COMMENT ON COLUMN public.document_approver.send_users_1 IS 'массив id пользователей, адресатов уведомления Руководителей ПД';



COMMENT ON COLUMN public.document_approver.send_date_2 IS 'Дата+время уведомления Экспертов ПД';



COMMENT ON COLUMN public.document_approver.send_users_2 IS 'массив id пользователей, адресатов уведомления Экспертов ПД';



COMMENT ON COLUMN public.document_approver.is_preapproved IS 'Индикатор согласования ПД вне АСЭЗ';



COMMENT ON COLUMN public.document_approver.is_removed IS 'Индикатор удаления записи';



COMMENT ON COLUMN public.document_approver.created_at IS 'Дата+время назначения  ПД  (создания записи)';



COMMENT ON COLUMN public.document_approver.changed_at IS 'Дата+время последнего изменения записи';



COMMENT ON COLUMN public.document_approver.created_by IS 'id пользователя, создавшего запись';



COMMENT ON COLUMN public.document_approver.changed_by IS 'id пользователя, изменившего запись';



ALTER TABLE ONLY public.document_approver
    ADD CONSTRAINT document_approver_pkey PRIMARY KEY (uuid);
