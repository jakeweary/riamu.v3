import logging
import rich.logging

class Formatter(logging.Formatter):
  def format(self, record: logging.LogRecord):
    level_color = [0, 4, 2, 3, 1, 5][record.levelno // 10]
    level = f'\x1b[3{level_color}m{record.levelname:>8}\x1b[0m'
    name = f'\x1b[2m{record.name}\x1b[0m'
    return f'{level} {name} {record.getMessage()}'

def init_formatted_stderr(filter: str = ''):
  handler = logging.StreamHandler()
  handler.addFilter(logging.Filter(filter))
  handler.setFormatter(Formatter())
  logging.basicConfig(handlers=[handler], level=logging.NOTSET)

def init_rich(filter: str = ''):
  handler = rich.logging.RichHandler()
  handler.addFilter(logging.Filter(filter))
  logging.basicConfig(handlers=[handler], level=logging.NOTSET, format='%(message)s', datefmt='[%X]')
