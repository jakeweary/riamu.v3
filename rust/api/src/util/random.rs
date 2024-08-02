use rand::prelude::*;

#[allow(dead_code)]
pub struct Weighted<T> {
  pub item: T,
  pub local_index: usize,
  pub local_total: usize,
  pub global_index: usize,
  pub global_total: usize,
}

pub fn weighted<'a, T, U, I, F, W>(items: I, filter: F, weight: W) -> Option<Weighted<&'a T>>
where
  U: IntoIterator<Item = &'a T>,
  I: Fn() -> U,
  F: Fn(&'a T) -> bool,
  W: Fn(&'a T) -> usize,
{
  let items = || items().into_iter().filter(|&t| filter(t));

  let global_total = items().map(&weight).sum();
  let global_index = match global_total {
    0 => return None,
    n => thread_rng().gen_range(0..n),
  };

  let (item, global_subtotal) = items()
    .scan(0, |acc, item| {
      *acc += weight(item);
      Some((item, *acc))
    })
    .find(|&(_, acc)| global_index < acc)
    .unwrap();

  let local_total = weight(item);
  let local_index = global_index + local_total - global_subtotal;

  tracing::debug! {
    "random pick: #{}/{} (#{}/{})",
    1 + global_index, global_total,
    1 + local_index, local_total,
  }

  Some(Weighted {
    item,
    global_total,
    global_index,
    local_total,
    local_index,
  })
}
