package modules;

use nginx;

sub touch_and_redirect {
  my $r = shift;
  if (-f $r->filename) {
    utime(undef, undef, $r->filename);
  }
  $r->internal_redirect('@sendfile');
  return OK;
}

1;
