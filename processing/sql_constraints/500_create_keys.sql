-- Test migrations do not guarantee an order, but
-- ALTER TABLE public.agenda_protocol_relation
--     ADD FOREIGN KEY (protocol_uuid) REFERENCES protocol(uuid) ON DELETE CASCADE,
--     ADD FOREIGN KEY (agenda_uuid) REFERENCES agenda(uuid) ON DELETE CASCADE;

-- ALTER TABLE public.item_relation_agenda_protocol
--     ADD FOREIGN KEY (protocol_uuid) REFERENCES protocol (uuid) ON DELETE CASCADE,
--     ADD FOREIGN KEY (agenda_uuid) REFERENCES agenda (uuid) ON DELETE CASCADE,
--     ADD FOREIGN KEY (protocol_item_uuid) REFERENCES protocol_item (uuid) ON DELETE CASCADE,
--     ADD FOREIGN KEY (agenda_item_uuid) REFERENCES agenda_item (uuid) ON DELETE CASCADE;

-- ALTER TABLE public.protocol_item
--   -- Требуется ли доп поведение?
--     ADD FOREIGN KEY (protocol_uuid) REFERENCES protocol (uuid) ON DELETE CASCADE,
--     ADD FOREIGN KEY (source_uuid) REFERENCES plan (uuid) ON DELETE CASCADE;

-- ALTER TABLE public.agenda_item
--     ADD FOREIGN KEY (agenda_uuid) REFERENCES agenda (uuid) ON DELETE CASCADE,
--     ADD FOREIGN KEY (source_uuid) REFERENCES plan (uuid) ON DELETE CASCADE;
