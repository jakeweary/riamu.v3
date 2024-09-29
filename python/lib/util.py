from typing import Callable, Hashable, Iterable

def unique_by[T](key: Callable[[T], Hashable], items: Iterable[T]):
  seen: set[Hashable] = set()
  unique: list[T] = []
  for item in items:
    k = key(item)
    if k not in seen:
      seen.add(k)
      unique.append(item)
  return unique
