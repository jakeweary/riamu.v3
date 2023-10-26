# this file is for quick testing purposes
# run with: `pipenv run python -m python`

from .lib import dl, dz, logs

logs.init_rich()

for query in ['Baby Shark', 'Despacito', 'Gangnam Style']:
  info = dl.ytsearch(query)
  print()
  print(f'\x1b[1mSearch results for \x1b[33m{query}\x1b[39m:\x1b[0m')
  for entry in info["entries"][:3]:
    print(f'- {entry["title"]}')
    print(f'  \x1b[34m{entry["url"]}\x1b[39m · \x1b[33m{entry["view_count"]:,}\x1b[39m views')
  print()

for query in ['Death Grips', 'Crystal Castles', 'Skeler']:
  info = dz.search(query)
  print()
  print(f'\x1b[1mSearch results for \x1b[33m{query}\x1b[39m:\x1b[0m')
  for entry in info['data'][:10]:
    print(f'- \x1b[33m{entry["title"]}\x1b[39m by \x1b[34m{entry["artist"]["name"]}\x1b[39m')
  print()
