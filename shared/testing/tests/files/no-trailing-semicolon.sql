drop table if exists no_trailing_semicolon;
create table no_trailing_semicolon(
    id integer
);

insert into no_trailing_semicolon values (10), (20), (30)
