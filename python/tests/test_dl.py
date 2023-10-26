import os
import tempfile

import pytest

from ..lib import dl

def test_search():
  info = dl.ytsearch('Skeler')
  assert len(info['entries']) != 0

def test_download(tmpdir: str):
  url = 'https://www.youtube.com/watch?v=buih7o5O0vk'
  dl.download(url, tmpdir)
  assert len(os.listdir(tmpdir)) == 1

@pytest.fixture()
def tmpdir():
  with tempfile.TemporaryDirectory() as path:
    yield path
