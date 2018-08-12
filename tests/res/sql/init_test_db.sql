BEGIN;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS countries;

CREATE TABLE countries (
  code       TEXT,
  name       TEXT,
  population INTEGER
);
CREATE TABLE test_types (
  t  TEXT, -- text affinity by rule 2
  nu NUMERIC, -- numeric affinity by rule 5
  i  INTEGER, -- integer affinity by rule 1
  r  REAL, -- real affinity by rule 4
  no BLOB      -- no affinity by rule 3
);

-- TODO: get real data (without duplicates)
insert into countries (code, name, population)
values ('HT', 'Haiti', 446);
insert into countries (code, name, population)
values ('CN', 'China', 953);
insert into countries (code, name, population)
values ('BR', 'Brazil', 999);
insert into countries (code, name, population)
values ('CN', 'China', 314);
insert into countries (code, name, population)
values ('TH', 'Thailand', 889);
insert into countries (code, name, population)
values ('UA', 'Ukraine', 491);
insert into countries (code, name, population)
values ('PH', 'Philippines', 836);
insert into countries (code, name, population)
values ('CN', 'China', 935);
insert into countries (code, name, population)
values ('IT', 'Italy', 975);
insert into countries (code, name, population)
values ('ID', 'Indonesia', 357);

COMMIT;
