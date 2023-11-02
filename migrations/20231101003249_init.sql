create table if not exists "counter" (
  "name" text not null primary key,
  "count" bigint not null default 0
);

create table if not exists "user" (
  "id" bigint not null primary key,
  "name" text,
  "messages" bigint not null default 0,
  "commands" bigint not null default 0
);

create table if not exists "status" (
  "id" integer not null primary key autoincrement,
  "user" bigint not null,
  "status" text not null,
  "time" bigint not null,
  "desktop" text,
  "mobile" text,
  "web" text,
  foreign key ("user") references "user" ("id")
);

--

insert or ignore into "counter" ("name")
values ('events'), ('messages'), ('commands');
