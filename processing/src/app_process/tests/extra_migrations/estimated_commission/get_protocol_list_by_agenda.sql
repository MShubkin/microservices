INSERT INTO agenda(uuid, id, meeting_date, status_id, pricing_organization_unit_id, is_removed, created_at, changed_at, created_by, changed_by) values
        ('00000000-0000-0000-0000-000000000001',1,'2000-01-01',200,2,false,'1900-01-01','1900-01-01',99,99);

INSERT INTO protocol(
    uuid,
    id,
    protocol_type_id,
    registration_number,
    status_id,
    pricing_organization_unit_id,
    is_secret,
    is_removed,
    created_by,
    changed_by,
    created_at,
    changed_at,
    protocol_date) values
        ('00000000-0000-0000-0000-000000000001','1',1,'111',100,1,false,false,99,99,'1900-01-01','1900-01-01','1911-11-11'),
        ('00000000-0000-0000-0000-000000000002','2',1,'222',200,1,false,false,99,99,'1900-01-01','1900-01-01','1911-11-12'),
        ('00000000-0000-0000-0000-000000000003','3',1,'333',300,1,false,false,99,99,'1900-01-01','1900-01-01','1911-11-13'),
        ('00000000-0000-0000-0000-000000000004','4',1,'444',400,1,false,false,99,99,'1900-01-01','1900-01-01','1911-11-14');

INSERT INTO agenda_protocol_relation (
    protocol_uuid,
    agenda_uuid,
    created_at,
    created_by) values 
       ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', now(), 1),
       ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', now(), 1),
       ('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000001', now(), 1),
       ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000001', now(), 1);
