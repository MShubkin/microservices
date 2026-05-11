drop table if exists semicolon_in_string;
create table semicolon_in_string (
  s varchar
);
insert into semicolon_in_string values ('some string with ; inside');
