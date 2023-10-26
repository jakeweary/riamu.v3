import logging
import rich.logging

class Formatter(logging.Formatter):
  def format(self, record: logging.LogRecord) -> str:
    level_color = [0, 4, 2, 3, 1, 5][record.levelno // 10]
    level = f'\x1b[3{level_color}m{record.levelname:>8}\x1b[0m'
    name = f'\x1b[30m{record.name}\x1b[0m'
    return f'{level} {name} {record.getMessage()}'

def init_formatted_stderr():
  handler = logging.StreamHandler()
  handler.setFormatter(Formatter())
  logging.basicConfig(handlers=[handler], level=logging.NOTSET)

def init_rich():
  handler = rich.logging.RichHandler()
  logging.basicConfig(handlers=[handler], level=logging.NOTSET, format='%(message)s', datefmt='[%X]')
