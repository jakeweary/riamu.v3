# stolen from:
# https://github.com/yt-dlp/yt-dlp/blob/master/devscripts/cli_to_api.py

# example usage:
# pipenv run python -m python.cli_to_api \
#   -vxf 'ba[ext=m4a]' -o '%(artist)s - %(title)s.%(ext)s' \
#   --embed-metadata --parse-metadata 'title:(?P<artist>.+) [-‐‑‒–—―] (?P<title>.+)' \
#   --embed-thumbnail --convert-thumbnails jpg \
#   --ppa 'ThumbnailsConvertor+ffmpeg_o:-q:v 1 -vf crop=iw:iw*9/16' \
#   --no-mtime https://youtu.be/buih7o5O0vk

# type: ignore

import sys

import rich
import rich.pretty
import yt_dlp
import yt_dlp.options

def parse_patched_options(opts):
  create_parser = yt_dlp.options.create_parser
  patched_parser = create_parser()
  patched_parser.defaults.update({
    'ignoreerrors': False,
    'retries': 0,
    'fragment_retries': 0,
    'extract_flat': False,
    'concat_playlist': 'never',
  })
  yt_dlp.options.__dict__['create_parser'] = lambda: patched_parser
  try:
    return yt_dlp.parse_options(opts)
  finally:
    yt_dlp.options.__dict__['create_parser'] = create_parser

def cli_to_api(opts, cli_defaults=False):
  opts = (yt_dlp.parse_options if cli_defaults else parse_patched_options)(opts).ydl_opts
  diff = {k: v for k, v in opts.items() if default_opts[k] != v}
  if 'postprocessors' in diff:
    diff['postprocessors'] = [pp for pp in diff['postprocessors'] if pp not in default_opts['postprocessors']]
  return diff

default_opts = parse_patched_options([]).ydl_opts

rich.print('The arguments passed translate to:')
rich.pretty.pprint(cli_to_api(sys.argv[1:]))

rich.print('Combining these with the CLI defaults gives:')
rich.pretty.pprint(cli_to_api(sys.argv[1:], True))
