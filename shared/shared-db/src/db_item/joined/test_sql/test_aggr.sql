DROP TABLE IF EXISTS plan_item;
DROP TABLE IF EXISTS plan_secret;

CREATE TABLE plan_item(id BIGINT NOT NULL, plan_id BIGINT NOT NULL, content TEXT NOT NULL);

CREATE TABLE plan_secret(id BIGINT NOT NULL, plan_id BIGINT NOT NULL, code TEXT NOT NULL, description TEXT NOT NULL);

INSERT INTO plan_item(id, plan_id, content) values
    (1, 1, 'item11'),
    (2, 1, 'item11'),
    (3, 1, 'item11'),
    (4, 1, 'item11'),
    (5, 1, 'item11'),
    (6, 1, 'item11');

INSERT INTO plan_secret(id, plan_id, code, description) values
    (1, 1, 'code1', 'desc1'),
    (2, 1, 'code2', 'desc2'),
    (3, 1, 'code3', 'desc3'),
    (4, 1, 'code4', 'desc4'),
    (5, 1, 'code5', 'desc5'),
    (6, 1, 'code6', 'desc6');
