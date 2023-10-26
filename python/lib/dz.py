import functools
import logging
import os
from typing import Any

from deemix import generateDownloadObject # type: ignore
from deemix.downloader import Downloader
from deemix.plugins.spotify import Spotify
from deemix.settings import DEFAULTS
from deemix.utils import getBitrateNumberFromText # type: ignore
from deemix.types.DownloadObjects import Single
from deezer import Deezer

logger = logging.getLogger(__name__)

# ---

class DzException(Exception):
  pass

def search(query: str) -> dict[str, Any]:
  dz = _dz()
  return dz.api.search_track(query) # type: ignore

def generate_download_object(url: str, bitrate: str = 'flac') -> Any:
  dz = _dz()
  br = getBitrateNumberFromText(bitrate)
  return generateDownloadObject(dz, url, br, dz.plugins, dz.listener) # type: ignore

def download(dl_obj: Any, out_dir: str) -> None:
  if not isinstance(dl_obj, Single):
    raise DzException("single tracks only")
  dz = _dz()
  settings = DEFAULTS | {'downloadLocation': out_dir}
  Downloader(dz, dl_obj, settings, dz.listener).start()

# ---

@functools.cache
def _dz():
  return _Deezer()

class _Deezer(Deezer):
  def __init__(self):
    super().__init__()
    self.listener = _LogListener()
    self.plugins = {'spotify': _Spotify()}
    self.login_via_arl(os.environ['DEEZER_ARL']) # type: ignore

class _Spotify(Spotify):
  def __init__(self):
    super().__init__() # type: ignore
    self.credentials = {
      'clientId': os.environ['SPOTIFY_APP_ID'],
      'clientSecret': os.environ['SPOTIFY_APP_SECRET'],
    }
    self.configFolder.mkdir(parents=True, exist_ok=True)
    self.checkCredentials()

class _LogListener:
  def send(self, key: str, value: Any = None):
    logger.debug(f'{key} {value}')
