from typing import Callable, Hashable, Iterable, TypeVar

_T = TypeVar('_T')

def unique_by(key: Callable[[_T], Hashable], items: Iterable[_T]):
  seen: set[Hashable] = set()
  unique: list[_T] = []
  for item in items:
    k = key(item)
    if k not in seen:
      seen.add(k)
      unique.append(item)
  return unique
