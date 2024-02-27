use serenity::all::*;

use super::*;

#[derive(Clone, Copy, sqlx::FromRow)]
pub struct Row {
  pub time: i64,
  pub status: Packed,
}

pub async fn query(pool: &Pool, uid: UserId, range: &str) -> sqlx::Result<Vec<Row>> {
  let q = sqlx::query_as(
    " with a as ( select * from statuses
                  where user = $1 and time < unixepoch('now', $2)
                  order by time desc limit 1 ),
           b as ( select * from statuses
                  where user = $1 and time > unixepoch('now', $2)
                  order by time asc )
      select time, packed as status from a union all
      select time, packed           from b ",
  );
  let uid = uid.get() as i64;
  q.bind(uid).bind(range).fetch_all(pool).await
}

pub async fn insert(pool: &Pool, uid: UserId, status: Status) -> sqlx::Result<QueryResult> {
  let q = sqlx::query(
    " insert or ignore into statuses (user, packed)
      values (?, ?) ",
  );
  let uid = uid.get() as i64;
  let packed = Packed::from(status);
  q.bind(uid).bind(packed).execute(pool).await
}

// ---

#[derive(Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct Packed(pub i64);

impl Packed {
  pub fn status(self) -> OnlineStatus {
    deserialize(self.0 % 10)
  }

  pub fn desktop(self) -> Option<OnlineStatus> {
    let n = self.0 / 10 % 10;
    (n != 0).then(|| deserialize(n))
  }

  pub fn mobile(self) -> Option<OnlineStatus> {
    let n = self.0 / 100 % 10;
    (n != 0).then(|| deserialize(n))
  }

  pub fn web(self) -> Option<OnlineStatus> {
    let n = self.0 / 1000 % 10;
    (n != 0).then(|| deserialize(n))
  }
}

impl From<Status> for Packed {
  fn from(s: Status) -> Self {
    let web = s.web.map_or(0, serialize);
    let mobile = s.mobile.map_or(0, serialize);
    let desktop = s.desktop.map_or(0, serialize);
    let status = serialize(s.status);
    Self(1000 * web + 100 * mobile + 10 * desktop + status)
  }
}

// ---

pub struct Status {
  pub status: OnlineStatus,
  pub desktop: Option<OnlineStatus>,
  pub mobile: Option<OnlineStatus>,
  pub web: Option<OnlineStatus>,
}

impl From<Packed> for Status {
  fn from(s: Packed) -> Self {
    Self {
      status: s.status(),
      desktop: s.desktop(),
      mobile: s.mobile(),
      web: s.web(),
    }
  }
}

impl From<&Presence> for Status {
  fn from(p: &Presence) -> Self {
    let (desktop, mobile, web) = match &p.client_status {
      Some(s) => (s.desktop, s.mobile, s.web),
      None => Default::default(),
    };

    Self {
      status: p.status,
      desktop,
      mobile,
      web,
    }
  }
}

// ---

fn serialize(s: OnlineStatus) -> i64 {
  match s {
    OnlineStatus::Offline => 1,
    OnlineStatus::Online => 2,
    OnlineStatus::Idle => 3,
    OnlineStatus::DoNotDisturb => 4,
    _ => panic!(),
  }
}

fn deserialize(n: i64) -> OnlineStatus {
  match n {
    1 => OnlineStatus::Offline,
    2 => OnlineStatus::Online,
    3 => OnlineStatus::Idle,
    4 => OnlineStatus::DoNotDisturb,
    _ => panic!(),
  }
}
