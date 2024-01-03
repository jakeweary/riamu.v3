alter table "counters" rename to "counters.old";
alter table "users" rename to "users.old";
alter table "statuses" rename to "statuses.old";
alter table "gcra" rename to "gcra.old";

--

create table "counters" (
  "name" text primary key,
  "count" integer not null default 0
) strict;

create table "users" (
  "id" integer primary key,
  "name" text,
  "messages" integer not null default 0,
  "commands" integer not null default 0
) strict, without rowid;

create table "statuses" (
  "time" integer default (unixepoch()),
  "user" integer references "users",

  -- stored as 4-digit number: wmds (web, mobile, desktop, status)
  -- 0 null, 1 offline, 2 online, 3 idle, 4 dnd
  "packed" integer,

  primary key ("time", "user", "packed")
) strict, without rowid;

create table "gcra" (
  "key" integer primary key,
  "tat" integer not null -- unix time in ns
) strict, without rowid;

--

insert into "counters" select "name", "count" from "counters.old";
insert into "users" select "id", "name", "messages", "commands" from "users.old";
insert into "statuses" select "time", "user", "packed" from "statuses.old";
insert into "gcra" select "key", "tat" from "gcra.old";

--

drop table "gcra.old";
drop table "statuses.old";
drop table "users.old";
drop table "counters.old";
