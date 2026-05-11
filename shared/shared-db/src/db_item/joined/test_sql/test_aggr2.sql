-- DROP TABLE IF EXISTS plan;
DROP TABLE IF EXISTS plan_item;
DROP TABLE IF EXISTS plan_owner;
DROP TABLE IF EXISTS addresses;
DROP TABLE IF EXISTS secrets;

CREATE TABLE plan_item(id BIGINT NOT NULL, plan_id BIGINT NOT NULL, content TEXT NOT NULL);

CREATE TABLE plan_owner(id BIGINT NOT NULL, plan_id BIGINT NOT NULL, name TEXT NOT NULL);

CREATE TABLE addresses(id BIGINT NOT NULL, object_id BIGINT NOT NULL, address TEXT NOT NULL);

CREATE TABLE secrets(id BIGINT NOT NULL, object_id BIGINT NOT NULL, code TEXT NOT NULL);



INSERT INTO plan_item(id, plan_id, content) values
    (1, 1, 'item11'),
    (2, 1, 'item11'),
    (3, 2, 'item11'),
    (4, 2, 'item11'),
    (5, 2, 'item11'),
    (6, 3, 'item11');

INSERT INTO plan_owner(id, plan_id, name) values
    (342,1,'Колобок'),
    (342,2,'Колобок'),
    (342,2,'Колобок'),
    (343,3,'Маленький Принц'),
    (343,3,'Маленький Принц'),
    (343,1,'Маленький Принц');

INSERT INTO secrets(id, object_id, code) values
    (1, 1, 'code1'),
    (2, 2, 'code2'),
    (3, 3, 'code3'),
    (4, 4, 'code4'),
    (5, 5, 'code5'),
    (6, 6, 'code6'),
    (7, 342, 'code10'),
    (8, 342, 'code20'),
    (9, 343, 'code30'),
    (10, 342, 'code40'),
    (11, 342, 'code50'),
    (12, 342, 'code60'),
    (13, 343, 'code200'),
    (14, 343, 'code300'),
    (15, 343, 'code400'),
    (16, 342, 'code500'),
    (17, 342, 'code600');

INSERT INTO addresses(id, object_id, address) values 
    (1, 342, 'У бабушки и дедушки'),
    (2, 342, 'Mars'),
    (3, 343, 'Asteroid Belt'),
    (4, 343, 'Earth'),
    (5, 344, 'In our hearts'),
    (6, 344, 'North Pole'),
    (7, 344, 'Your chimney');
