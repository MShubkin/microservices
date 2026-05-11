TRUNCATE TABLE public.protocol;

INSERT INTO public.protocol ("uuid",id,protocol_type_id,registration_number,status_id,pricing_organization_unit_id,is_secret,is_removed,protocol_date,created_at,changed_at,created_by,changed_by) VALUES
	 ('00000017-0000-0000-0092-000000000099'::uuid,8900000000,1,NULL,100,1,false,false,'1901-01-01','2024-09-17 12:02:14.01432','2024-09-17 12:02:14.01432',666,666),
	 ('00000017-0000-0000-0030-000000000099'::uuid,8900000001,1,NULL,100,0,false,false,'1901-01-01','2024-09-18 12:25:38.387545','2024-09-18 12:25:38.387545',666,666),
	 ('00000017-0000-0000-0080-000000000099'::uuid,8900000001,1,NULL,100,0,false,false,'1901-01-01','2024-09-18 12:25:38.387545','2024-09-18 12:25:38.387545',666,666);
