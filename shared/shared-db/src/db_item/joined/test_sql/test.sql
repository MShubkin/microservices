DROP TABLE IF EXISTS plan_item;
DROP TABLE IF EXISTS plan_secret;
DROP TABLE IF EXISTS plan_owner;
DROP TABLE IF EXISTS addresses;

CREATE TABLE plan_item(id BIGINT NOT NULL, plan_id BIGINT NOT NULL, content TEXT NOT NULL);

CREATE TABLE plan_secret(id BIGINT NOT NULL, plan_id BIGINT NOT NULL, code TEXT NOT NULL, description TEXT NOT NULL);

CREATE TABLE plan_owner(id BIGINT NOT NULL, plan_id BIGINT NOT NULL, name TEXT NOT NULL, address_id BIGINT NOT NULL);

CREATE TABLE addresses(id BIGINT NOT NULL, address TEXT NOT NULL);

INSERT INTO plan_item(id, plan_id, content) values
    (1,5,'stuffing'),
    (2,5,'awdadw'),
    (3,5,'another entry'),
    (3,7,'ho ho ho');

INSERT INTO plan_secret(id, plan_id, code, description) values
    (999,5,'006','I dislike James Bond'),
    (1,3,'007','For me');

INSERT INTO plan_owner(id, plan_id, name, address_id) values
    (342,5,'Колобок',1),
    (343,7,'Маленький Принц',3),
    (343,5,'Маленький Принц',2),
    (343,5,'Маленький Принц',4),
    (344,7,'Дед Мороз',7),
    (344,5,'Дед Мороз',5),
    (344,7,'Дед Мороз',6);

INSERT INTO addresses(id, address) values 
    (1, 'У бабушки и дедушки'),
    (2, 'Mars'),
    (3,'Asteroid Belt'),
    (4,'Earth'),
    (5,'In our hearts'),
    (6,'North Pole'),
    (7,'Your chimney');
