import logging
from operator import itemgetter
from typing import Any, Callable, Optional

from yt_dlp import YoutubeDL

from . import util

logger = logging.getLogger(__name__)

# ---

Params = dict[str, Any]
Info = dict[str, Any]
Format = dict[str, Any]
FormatId = str
FormatSelectorContext = dict[str, Any]
FormatSelector = Callable[[FormatSelectorContext], list[FormatId]]

def download(url: str, out_dir: str, fs: Optional[FormatSelector] = None, **kwargs: Any) -> Info:
  format = _wrap_format_selector(fs) if fs else {}
  params = _defaults() | {'paths': {'home': out_dir}} | format | kwargs
  with YoutubeDL(params) as ydl:
    return ydl.extract_info(url) # type: ignore

def ytsearch(query: str, limit: int = 50, **kwargs: Any) -> Info:
  params = _defaults() | {'extract_flat': 'in_playlist'} | kwargs
  with YoutubeDL(params) as ydl:
    full_query = f'ytsearch{limit}:{query}'
    info = ydl.extract_info(full_query, download=False, process=False) # type: ignore
    return _fix_info(info) # type: ignore

# ---

def _defaults() -> Params:
  return {
    'verbose': True,
    'logger': _Logger(),
    'restrictfilenames': True,
    'concurrent_fragment_downloads': 16,
    # 'external_downloader': 'aria2c',
    # 'external_downloader_args': ['-q', '-k1M'],
    'postprocessors': [
      {'key': 'FFmpegMetadata'},
    ],
  } # yapf: disable (https://github.com/google/yapf/issues/1015)

def _wrap_format_selector(fs: FormatSelector) -> Params:
  info_ref: Info = {}

  def match_filter(info: Info, *, incomplete: Any):
    nonlocal info_ref
    info_ref = info

  def format(ctx: FormatSelectorContext):
    formats_dict = {fmt['format_id']: fmt for fmt in ctx['formats']}
    formats = [formats_dict[fmt_id] for fmt_id in fs(info_ref | ctx)]
    yield _merge_formats(formats) if len(formats) > 1 else formats[0]

  return {'match_filter': match_filter, 'format': format}

def _merge_formats(formats: list[Format]) -> Format:
  # https://github.com/yt-dlp/yt-dlp#use-a-custom-format-selector
  return {
    'requested_formats': formats,
    'format_id': '+'.join(f['format_id'] for f in formats),
    'protocol': '+'.join(f['protocol'] for f in formats),
    'ext': formats[0]['ext'],
  }

def _fix_info(info: Info) -> Info:
  entries = util.unique_by(itemgetter('url'), info['entries'])
  entries.sort(key=lambda e: e['channel'] is None)
  return info | {'entries': entries}

class _Logger:
  def debug(self, msg: str) -> None:
    if msg.startswith('[debug] '):
      return self.debug(msg[8:])
    if msg.startswith('[info] '):
      return self.info(msg[7:])
    logger.debug(msg)

  def info(self, msg: str) -> None:
    logger.info(msg)

  def warning(self, msg: str) -> None:
    logger.warning(msg)

  def error(self, msg: str) -> None:
    logger.error(msg)
