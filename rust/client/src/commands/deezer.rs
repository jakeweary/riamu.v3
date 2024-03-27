use std::fmt::Write;
use std::fs;
use std::time::Duration;

use ::cache::Name;
use discord::link::{self, Link};
use fmt::num::Format as _;
use futures::StreamExt;
use python::lib::dz;
use serenity::all::*;
use url::Url;
use util::task;

use crate::client::{err, Context, Result};

#[macros::command(desc = "Download a song from Deezer (tries to upload it directly to Discord)")]
pub async fn as_file(ctx: &Context<'_>, #[desc = "A search query or a Deezer/Spotify link"] query: &str) -> Result<()> {
  deezer(ctx, query, false).await
}

#[macros::command(desc = "Download a song from Deezer (gives a direct link and a nice looking banner)")]
pub async fn as_direct_link(
  ctx: &Context<'_>,
  #[desc = "A search query or a Deezer/Spotify link"] query: &str,
) -> Result<()> {
  deezer(ctx, query, true).await
}

async fn deezer(ctx: &Context<'_>, query: &str, with_banner: bool) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("converting query to url…");
  let url = query_to_url(ctx, query).await?;
  tracing::debug!(%url, "selected");

  tracing::debug!("selecting track quality…");
  let quality = select_track_quality(ctx).await?;
  tracing::debug!(%quality, "selected");

  tracing::debug!("downloading…");
  ctx.progress("downloading…").await?;
  let tempdir = tempfile::tempdir()?;
  let info = {
    let tempdir = tempdir.path().to_owned();
    let info = task::spawn_blocking(move || dz::download(&url, &quality, &tempdir)).await?;
    info? // FIXME: should figure out a way to handle that PyErr
  };

  let Some(Ok(file)) = fs::read_dir(&tempdir)?.next() else {
    err::message!("failed to download");
  };
  let fpath = file.path();
  let fext = fpath.extension().and_then(|e| e.to_str()).unwrap();
  let fname = fpath.file_name().unwrap().to_string_lossy();
  let fsize = fpath.metadata()?.len();
  tracing::debug!(file = ?fname, "downloaded {}B", fsize.iec());

  tracing::debug!("caching…");
  let url = {
    let fpath = fpath.clone();
    let fname = Name::Set(format!("{} - {}.{}", info.artist.name, info.title, fext));
    ctx.client.cache.store_file(fpath, fname).await?.unwrap()
  };

  if with_banner {
    let content = {
      let mut acc = String::new();
      write!(acc, "[track](<https://deezer.com/track/{}>) ", info.id)?;
      write!(acc, "[artist](<https://deezer.com/artist/{}>) ", info.artist.id)?;
      write!(acc, "[album](<https://deezer.com/album/{}>) ", info.album.id)?;
      write!(acc, "\u{205D} {} {}B", Link(fext, url.as_str()), fsize.iec())?;
      acc
    };

    tracing::debug!("rendering banner…");
    let banner = task::spawn_blocking(move || -> Result<_> {
      let mut png = Vec::new();
      banner::render(&fpath)?.write_to_png(&mut png)?;
      Ok(png)
    })
    .await??;

    let banner = CreateAttachment::bytes(banner, "banner.png");
    let edit = EditInteractionResponse::new()
      .components(Default::default()) // remove components
      .content(content)
      .new_attachment(banner);

    tracing::debug!("sending response…");
    ctx.event.edit_response(ctx, edit).await?;
  } else {
    let content = {
      let (artist, track) = (link::Name(&info.artist.name), link::Name(&info.title));
      let (artist_id, track_id) = (info.artist.id, info.id);
      let mut acc = String::new();
      write!(acc, "[{}](<https://deezer.com/artist/{}>) \u{2013} ", artist, artist_id)?;
      write!(acc, "[{}](<https://deezer.com/track/{}>) ", track, track_id)?;
      write!(acc, "\u{205D} {} {}B", Link(fext, url.as_str()), fsize.iec())?;
      acc
    };

    let edit = EditInteractionResponse::new()
      .components(Default::default()) // remove components
      .content(content);

    if fsize > ctx.filesize_limit().await? {
      tracing::debug!("sending response…");
      ctx.event.edit_response(ctx, edit).await?;
    } else {
      let file = CreateAttachment::path(&fpath).await?;
      let edit = edit.new_attachment(file);

      tracing::debug!("uploading…");
      ctx.progress("uploading…").await?;
      if ctx.event.edit_response(ctx, edit).await.is_err() {
        err::message!("failed to upload, most likely the file is too big ({fsize}B)");
      }
    }
  }

  Ok(())
}

async fn query_to_url(ctx: &Context<'_>, query: &str) -> Result<String> {
  let query = query.to_owned();
  match Url::parse(&query) {
    Ok(_) => Ok(query),
    Err(_) => {
      tracing::debug!("searching…");
      let tracks = task::spawn_blocking(move || dz::search(&query)).await??;
      tracing::debug!("found {} tracks", tracks.len());

      if tracks.is_empty() {
        err::message!("could not find anything");
      }

      tracing::debug!("selecting track url…");
      let url = select_track_url(ctx, &tracks).await?;
      Ok(url)
    }
  }
}

async fn select_track_url(ctx: &Context<'_>, tracks: &[dz::Track]) -> Result<String> {
  if tracks.len() == 1 {
    return Ok(tracks[0].link.clone());
  }

  let selector = CreateActionRow::SelectMenu({
    let options = tracks.iter().take(25).map(|e| {
      let label = format!("{} ({})", e.title, fmt::duration(e.duration));
      let desc = format!("{} · {}", e.artist.name, e.album.title);

      let label = fmt::ellipsis(&label, 100);
      let desc = fmt::ellipsis(&desc, 100);
      CreateSelectMenuOption::new(label, &e.link).description(desc)
    });

    let options = options.collect();
    let menu = CreateSelectMenuKind::String { options };
    let menu = CreateSelectMenu::new("url", menu);
    menu.placeholder("Select track")
  });

  let components = vec![selector];
  let edit = EditInteractionResponse::new().components(components);
  let msg = ctx.event.edit_response(ctx, edit).await?;

  tracing::debug!("waiting for user interaction…");
  let mut collector = msg
    .await_component_interaction(ctx)
    .author_id(ctx.event.user.id)
    .timeout(Duration::from_secs(60))
    .stream();

  let Some(mut mci) = collector.next().await else {
    err::timeout!();
  };

  mci.defer(ctx).await?;

  let url = match &mut mci.data.kind {
    ComponentInteractionDataKind::StringSelect { values } => values.swap_remove(0),
    _ => unreachable!(),
  };

  Ok(url)
}

async fn select_track_quality(ctx: &Context<'_>) -> Result<String> {
  let buttons = CreateActionRow::Buttons({
    let values = [("lossless", "lossless"), ("320", "320k"), ("128", "128k")];
    let buttons = values.into_iter().enumerate().map(|(i, (id, label))| {
      let style = match i {
        0 => ButtonStyle::Primary,
        _ => ButtonStyle::Secondary,
      };
      CreateButton::new(id).label(label).style(style)
    });
    buttons.collect()
  });

  let components = vec![buttons];
  let edit = EditInteractionResponse::new().components(components);
  let msg = ctx.event.edit_response(ctx, edit).await?;

  tracing::debug!("waiting for user interaction…");
  let mut collector = msg
    .await_component_interaction(ctx)
    .author_id(ctx.event.user.id)
    .timeout(Duration::from_secs(60))
    .stream();

  let Some(mci) = collector.next().await else {
    err::timeout!();
  };

  mci.defer(ctx).await?;

  Ok(mci.data.custom_id)
}

// ---

mod banner {
  use std::ffi::OsStr;
  use std::fmt::Write;
  use std::io::Cursor;

  use cairo::*;
  use cairo_ext::{ContextExt, ImageSurfaceExt};
  use pango::{Alignment, EllipsizeMode};
  use pangocairo::{prelude::*, FontMap};

  const SCALE: i32 = 8;
  const PADDING: i32 = 15;
  const IMAGE_W: i32 = 550;
  const IMAGE_H: i32 = 100 + 2 * PADDING;

  pub fn render(path: impl AsRef<OsStr>) -> super::Result<ImageSurface> {
    let img = ImageSurface::create(Format::Rgb24, SCALE * IMAGE_W, SCALE * IMAGE_H)?;
    let cc = Context::new(&img)?;
    cc.scale(SCALE as f64, SCALE as f64);
    cc.set_source_rgb_u32(0x1e1f22);
    cc.paint()?;

    let cover = ffmpeg::album_cover(&path, "png")?;
    let cover = ImageSurface::create_from_png(&mut Cursor::new(cover))?;

    let background = {
      let mut img = ImageSurface::create(Format::Rgb24, IMAGE_W, IMAGE_H + 20)?;

      let cc = Context::new(&img)?;
      cc.translate(img.width() as f64 / 2.0, img.height() as f64 / 2.0);
      cc.scale1(img.width() as f64 / cover.width() as f64);
      cc.translate(-cover.width() as f64 / 2.0, -cover.height() as f64 / 2.0);
      cc.set_source_surface(&cover, 0.0, 0.0)?;
      cc.paint()?;
      drop(cc);

      img.gaussian_blur(1.5)?;

      img
    };

    {
      cc.save()?;

      cc.set_source_surface(&background, 0.0, -10.0)?;
      cc.set_operator(Operator::Overlay);
      cc.paint_with_alpha(1.0 / 3.0)?;

      cc.translate(PADDING as f64, PADDING as f64);
      cc.scale1((IMAGE_H - PADDING * 2) as f64 / cover.width() as f64);
      cc.set_source_surface(&cover, 0.0, 0.0)?;
      cc.set_operator(Operator::Over);
      cc.paint()?;

      cc.restore()?;
    }

    {
      let fm = FontMap::default();
      let pc = fm.create_context();
      let layout = pango::Layout::new(&pc);
      layout.set_width((IMAGE_W - IMAGE_H - PADDING) * pango::SCALE);
      layout.set_auto_dir(false);

      let meta = ffmpeg::meta(&path)?;
      let title = meta.format.tag_or_empty(&["TITLE", "title"]);
      let artist = meta.format.tag_or_empty(&["ARTIST", "artist"]);
      let album = meta.format.tag_or_empty(&["ALBUM", "album"]);
      let genre = meta.format.tag(&["GENRE", "genre"]);
      let date = meta.format.tag(&["DATE", "date"]);
      let length = meta.format.tag(&["LENGTH", "TLEN"]);

      let footer = {
        let sep = " · ";
        let mut acc = String::new();
        if let Some(genre) = genre {
          write!(acc, "{}{}", genre.replace(';', ", "), sep)?;
        }
        if let Some(date) = date {
          write!(acc, "{}{}", &date[..4], sep)?;
        }
        if let Some(length) = length {
          write!(acc, "{}{}", fmt::duration(length.parse::<u64>()? / 1000), sep)?;
        }
        acc.truncate(acc.len().saturating_sub(sep.len()));
        acc
      };

      let markup = format! {
        concat! {
          "<span font='{}, sans'>",
            "<span font='@wght=400' size='14pt' color='#ffffffff'>{}</span>\n",
            "<span font='@wght=500' size='10pt' color='#ffffffbb'>{}</span>\n",
            "<span font='@wght=500' size='10pt' color='#ffffff77'>{}</span>",
          "</span>",
        },
        NOTO_SANS,
        glib::markup_escape_text(title),
        glib::markup_escape_text(&artist.replace(';', ", ")),
        glib::markup_escape_text(if [title, artist].contains(&album) { "" } else { album }),
      };

      cc.save()?;
      cc.translate(IMAGE_H as f64, (PADDING - 6) as f64);
      layout.set_alignment(Alignment::Left);
      layout.set_ellipsize(EllipsizeMode::End);
      layout.set_markup(&markup);
      pangocairo::update_layout(&cc, &layout);
      pangocairo::show_layout(&cc, &layout);
      cc.restore()?;

      let markup = format! {
        concat! {
          "<span font='{}, sans'>",
            "<span font='@wght=500' size='10pt' color='#ffffff77'>{}</span>",
          "</span>",
        },
        NOTO_SANS,
        glib::markup_escape_text(&footer),
      };

      cc.save()?;
      cc.translate(IMAGE_H as f64, (IMAGE_H - PADDING - 15) as f64);
      layout.set_alignment(Alignment::Right);
      layout.set_ellipsize(EllipsizeMode::Start);
      layout.set_markup(&markup);
      pangocairo::update_layout(&cc, &layout);
      pangocairo::show_layout(&cc, &layout);
      cc.restore()?;
    }

    Ok(img)
  }

  // ---

  macro_rules! font_family(($name:expr, $($suffix:expr),+) => {
    concat!($name, $(", ", $name, " ", $suffix),+)
  });

  // fc-query -f '%{family}\n' *.ttf | sort -u
  const NOTO_SANS: &str = font_family! {
    "Noto Sans", "Adlam", "Arabic UI", "Armenian", "Balinese", "Bamum",
    "Bassa Vah", "Bengali UI", "Canadian Aboriginal", "Cham", "Cherokee",
    "Devanagari", "Ethiopic", "Georgian", "Gujarati", "Gunjala Gondi",
    "Gurmukhi UI", "Hanifi Rohingya", "Hebrew", "Javanese", "Kannada UI",
    "Kawi", "Kayah Li", "Khmer UI", "Lao UI", "Lisu", "Malayalam UI",
    "Medefaidrin", "Meetei Mayek", "Myanmar", "Nag Mundari", "New Tai Lue",
    "Ol Chiki", "Oriya", "Sinhala UI", "Sora Sompeng", "Sundanese", "Syriac",
    "Syriac Eastern", "Tai Tham", "Tamil UI", "Tangsa", "Telugu UI", "Thaana",
    "Thai UI", "Vithkuqi", "HK", "JP", "KR", "SC", "TC", "Symbols"
  };
}
