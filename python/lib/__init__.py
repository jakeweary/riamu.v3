import importlib.metadata
import sys

def versions() -> list[tuple[str, str]]:
  acc: list[tuple[str, str]] = []
  ver, _, _ = sys.version.partition(' ')
  acc.append(('python', ver))
  for pkg in ['yt-dlp', 'gallery-dl', 'deemix', 'spotipy']:
    ver = importlib.metadata.version(pkg)
    acc.append((pkg, ver))
  return acc
