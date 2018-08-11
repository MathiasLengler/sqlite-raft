BEGIN;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS countries;

CREATE TABLE users (first_name TEXT, last_name TEXT, age INTEGER);
CREATE TABLE countries (
  code TEXT,
  name TEXT,
  population INTEGER
);

insert into users (first_name, last_name, age) values ('Ugo', 'Bernollet', 27);
insert into users (first_name, last_name, age) values ('Maxine', 'Headly', 31);
insert into users (first_name, last_name, age) values ('Regina', 'Dowdam', 10);
insert into users (first_name, last_name, age) values ('Larissa', 'Tortoise', 28);
insert into users (first_name, last_name, age) values ('Yankee', 'Fitzsimons', 86);
insert into users (first_name, last_name, age) values ('Julie', 'Whall', 34);
insert into users (first_name, last_name, age) values ('Hewitt', 'Rickardsson', 81);
insert into users (first_name, last_name, age) values ('Rowland', 'Gander', 83);
insert into users (first_name, last_name, age) values ('Dory', 'Grolmann', 30);
insert into users (first_name, last_name, age) values ('Vina', 'Andreev', 18);

insert into countries (code, name, population) values ('HT', 'Haiti', 446);
insert into countries (code, name, population) values ('CN', 'China', 953);
insert into countries (code, name, population) values ('BR', 'Brazil', 999);
insert into countries (code, name, population) values ('CN', 'China', 314);
insert into countries (code, name, population) values ('TH', 'Thailand', 889);
insert into countries (code, name, population) values ('UA', 'Ukraine', 491);
insert into countries (code, name, population) values ('PH', 'Philippines', 836);
insert into countries (code, name, population) values ('CN', 'China', 935);
insert into countries (code, name, population) values ('IT', 'Italy', 975);
insert into countries (code, name, population) values ('ID', 'Indonesia', 357);

COMMIT;
