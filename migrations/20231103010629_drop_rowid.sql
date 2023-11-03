alter table "users" rename to "users.old";
alter table "statuses" rename to "statuses.old";

--

create table "users" (
  "id" integer not null primary key,
  "name" text,
  "messages" integer not null default 0,
  "commands" integer not null default 0
) strict, without rowid;

create table "statuses" (
  "time" integer not null default (unixepoch()),
  "user" integer not null references "users",

  -- stored as 4-digit number: WMDS (Web, Mobile, Desktop, Status)
  -- 0 null, 1 offline, 2 online, 3 idle, 4 dnd
  "packed" integer not null,

  primary key ("time", "user", "packed")
) strict, without rowid;

--

insert into "users"
select "id", "name", "messages", "commands" from "users.old";

insert into "statuses"
select "time", "user", "packed" from "statuses.old";

--

drop table "statuses.old";
drop table "users.old";
