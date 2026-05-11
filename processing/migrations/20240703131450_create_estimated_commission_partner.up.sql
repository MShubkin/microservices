

CREATE TABLE public.estimated_commission_partner (
    uuid uuid NOT NULL,
    protocol_agenda_uuid uuid NOT NULL,
    user_id integer NOT NULL,
    e_mail character varying(255),
    is_checked_in boolean DEFAULT false NOT NULL,
    is_removed boolean DEFAULT false NOT NULL,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    created_by integer NOT NULL,
    changed_by integer NOT NULL,
    role_id smallint NOT NULL
);

ALTER TABLE ONLY public.estimated_commission_partner
    ADD CONSTRAINT estimated_commission_partner_pkey PRIMARY KEY (uuid);


COMMENT ON TABLE public.estimated_commission_partner IS 'Участники к Повестке/Протоколу';



COMMENT ON COLUMN public.estimated_commission_partner.uuid IS 'Уникальный идентификатор записи таблицы';



COMMENT ON COLUMN public.estimated_commission_partner.protocol_agenda_uuid IS 'Уникальный идентификатор Повестки/Протокола';



COMMENT ON COLUMN public.estimated_commission_partner.user_id IS 'Код члена СК';



COMMENT ON COLUMN public.estimated_commission_partner.e_mail IS 'E-mail члена СК. Автоматически определяется на основе данных пользователя, указанных в системе. 
Может быть вручную изменен пользователем на экране';



COMMENT ON COLUMN public.estimated_commission_partner.is_checked_in IS 'Признак присутствия. Заполняется пользователем вручную на экране';



COMMENT ON COLUMN public.estimated_commission_partner.is_removed IS 'Признак удаления записи';



COMMENT ON COLUMN public.estimated_commission_partner.created_at IS 'Дата и время создания.
Автоматически присваивается при создании объекта';



COMMENT ON COLUMN public.estimated_commission_partner.changed_at IS 'Дата и время изменения.
Автоматически присваивается при создании объекта';



COMMENT ON COLUMN public.estimated_commission_partner.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';



COMMENT ON COLUMN public.estimated_commission_partner.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
