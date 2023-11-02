create table "counters" (
  "name" text not null primary key,
  "count" integer not null default 0
) strict;

create table "users" (
  "id" integer not null primary key,
  "name" text,
  "messages" integer not null default 0,
  "commands" integer not null default 0
) strict;

create table "statuses" (
  "time" integer not null default (unixepoch()),
  "user" integer not null,

  -- stored as 4-digit number: WMDS (Web, Mobile, Desktop, Status)
  -- 0 null, 1 offline, 2 online, 3 idle, 4 dnd
  "packed" integer not null,

  unique ("time", "user", "packed"),
  foreign key ("user") references "users" ("id")
) strict;

--

insert into "counters" ("name", "count")
select "name", "count" from "counter";

insert into "users" ("id", "messages", "commands", "name")
select "id", "messages", "commands", "name" from "user";

insert or ignore into "statuses" ("time", "user", "packed")
select
  "time",
  "user",
  case "web"     when 'dnd' then 4 when 'idle' then 3 when 'online' then 2 when 'offline' then 1 else 0 end * 1000 +
  case "mobile"  when 'dnd' then 4 when 'idle' then 3 when 'online' then 2 when 'offline' then 1 else 0 end *  100 +
  case "desktop" when 'dnd' then 4 when 'idle' then 3 when 'online' then 2 when 'offline' then 1 else 0 end *   10 +
  case "status"  when 'dnd' then 4 when 'idle' then 3 when 'online' then 2 when 'offline' then 1 else 0 end
  as "packed"
from "status";

--

drop table "status";
drop table "user";
drop table "counter";
