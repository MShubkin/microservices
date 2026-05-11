TRUNCATE TABLE public.agenda_item;

INSERT INTO public.agenda_item ("uuid",agenda_uuid,source_uuid,"number",is_registered_by_d647,is_excluded,is_removed,reviewed_at,sum_excluded_vat,pricing_sum_excluded_vat,created_at,changed_at,created_by,changed_by) VALUES
	 ('60e613c1-ca44-42d4-92ac-82ad7ed87081'::uuid,'1e7dd81b-fcd4-4602-aee3-449d4806086a'::uuid,'8f93077a-11b9-401c-93dd-0487e06c93ee'::uuid,1,false,false,false,NULL,NULL,NULL,'2024-09-17 15:39:08.832164','2024-09-17 15:39:08.832164',658,658),
	 ('14496278-9077-4438-bf4c-cdd0beac1f25'::uuid,'1e7dd81b-fcd4-4602-aee3-449d4806086a'::uuid,'c192544d-e20a-44b3-99f7-a1578b23bad7'::uuid,2,false,false,false,NULL,NULL,NULL,'2024-09-17 15:39:08.83217','2024-09-17 15:39:08.83217',658,658),
	 ('2742c55e-c6de-4edd-a663-380477504b70'::uuid,'aedaf136-eee1-4893-a25a-750ce34a856f'::uuid,'2d170cc4-acb0-4f8b-8a81-204255bbf0e1'::uuid,1,false,false,false,NULL,NULL,NULL,'2024-09-17 15:54:26.075019','2024-09-17 15:54:26.075019',658,658),
	 ('d241083e-caf3-4d6f-9002-751ca3f844a3'::uuid,'1d0cb357-cda7-4d85-aa79-2ef5967c2d5d'::uuid,'3eda4de3-b578-49ab-aee8-516b1690a386'::uuid,1,false,false,false,NULL,NULL,NULL,'2024-09-18 12:08:29.783517','2024-09-18 12:08:29.783517',658,658),
	 ('0b7e13ed-b363-448d-904f-5707ec0ce2a9'::uuid,'1d0cb357-cda7-4d85-aa79-2ef5967c2d5d'::uuid,'953c5449-094d-40f9-8731-3964c68d9fac'::uuid,2,false,false,false,NULL,NULL,NULL,'2024-09-18 12:08:29.783524','2024-09-18 12:08:29.783524',658,658),
	 ('84b120f1-81e0-45a5-a24a-e2243675230e'::uuid,'10ea4b12-d563-434a-82e1-ea6ba6a31d9d'::uuid,'39137cba-8173-460c-b378-0a205489ca95'::uuid,1,false,false,false,NULL,NULL,NULL,'2024-09-18 12:19:14.582234','2024-09-18 12:19:14.582234',658,658),
	 ('7d6683b2-4352-4f0f-bc93-79a3496f8bef'::uuid,'10ea4b12-d563-434a-82e1-ea6ba6a31d9d'::uuid,'08483814-4f69-4f33-84c0-0f224132bb1c'::uuid,2,false,false,false,NULL,NULL,NULL,'2024-09-18 12:19:14.582236','2024-09-18 12:19:14.582236',658,658);
