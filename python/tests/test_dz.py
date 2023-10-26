from ..lib import dz

def test_search():
  info = dz.search('Skeler')
  assert len(info['data']) != 0

def test_direct_link():
  url = 'https://www.deezer.com/track/894382952'
  dl_obj = dz.generate_download_object(url)
  assert dl_obj.artist == 'Skeler'
  assert dl_obj.title == 'Pale Light'

def test_spotify_plugin():
  url = 'https://open.spotify.com/track/64F0tid5vwapfuC4ERAHyA'
  dl_obj = dz.generate_download_object(url)
  assert dl_obj.artist == 'Skeler'
  assert dl_obj.title == 'Pale Light'
