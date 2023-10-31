use std::ffi::OsStr;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::Cursor;

use cairo::glib;
use lib::cairo::blur::gaussian_blur;
use lib::cairo::ext::ContextExt;
use lib::{ffmpeg, fmt};
use pangocairo::prelude::FontMapExt;
use rayon::prelude::*;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

const SCALE: i32 = 1;
const PADDING: i32 = 15;
const IMAGE_W: i32 = 550;
const IMAGE_H: i32 = 100 + 2 * PADDING;

fn main() -> Result<()> {
  // rayon::ThreadPoolBuilder::new().num_threads(4).build_global()?;

  c::fontconfig::add_dir("assets/fonts")?;

  fs::read_dir("/mnt/d/Music/Random")?
    .par_bridge()
    .map(|entry| {
      let entry = entry?;
      let meta = entry.metadata()?;
      let path = entry.path();

      if meta.is_file() && path.extension().is_some_and(|ext| ext == "flac") {
        let name = path.file_name().unwrap().to_string_lossy();
        println!("{}", name);

        let img = render(&path)?;
        let path = format!("_/banners/{}.png", name);
        let mut file = File::create(path)?;
        img.write_to_png(&mut file)?;
      }

      Result::Ok(())
    })
    .try_for_each(|res| res)?;

  Ok(())
}

fn render(path: impl AsRef<OsStr>) -> Result<cairo::ImageSurface> {
  let img = cairo::ImageSurface::create(cairo::Format::Rgb24, SCALE * IMAGE_W, SCALE * IMAGE_H)?;
  let cc = cairo::Context::new(&img)?;
  cc.scale(SCALE as f64, SCALE as f64);
  cc.set_source_rgb_u32(0x1e1f22);
  cc.paint()?;

  let cover_png = ffmpeg::album_cover_png(&path)?;
  let cover = cairo::ImageSurface::create_from_png(&mut Cursor::new(cover_png))?;

  let background = {
    let mut img = cairo::ImageSurface::create(cairo::Format::Rgb24, IMAGE_W, IMAGE_H + 20)?;
    let cc = cairo::Context::new(&img)?;

    cc.translate(img.width() as f64 / 2.0, img.height() as f64 / 2.0);
    cc.scale1(img.width() as f64 / cover.width() as f64);
    cc.translate(-cover.width() as f64 / 2.0, -cover.height() as f64 / 2.0);
    cc.set_source_surface(&cover, 0.0, 0.0)?;
    cc.paint()?;

    drop(cc);
    gaussian_blur(&mut img, 1.5)?;

    img
  };

  {
    cc.save()?;

    cc.set_source_surface(&background, 0.0, -10.0)?;
    cc.set_operator(cairo::Operator::Overlay);
    cc.paint_with_alpha(1.0 / 3.0)?;

    cc.translate(PADDING as f64, PADDING as f64);
    cc.scale1((IMAGE_H - PADDING * 2) as f64 / cover.width() as f64);
    cc.set_source_surface(&cover, 0.0, 0.0)?;
    cc.set_operator(cairo::Operator::Over);
    cc.paint()?;

    cc.restore()?;
  }

  {
    let fm = pangocairo::FontMap::default();
    let pc = fm.create_context();
    let layout = pango::Layout::new(&pc);
    layout.set_width((IMAGE_W - IMAGE_H - PADDING) * pango::SCALE);
    layout.set_auto_dir(false);

    let meta = ffmpeg::meta(&path)?;
    let title = meta.tags.get_or_empty(&["TITLE", "title"]);
    let artist = meta.tags.get_or_empty(&["ARTIST", "artist"]);
    let album = meta.tags.get_or_empty(&["ALBUM", "album"]);
    let genre = meta.tags.get(&["GENRE", "genre"]);
    let date = meta.tags.get(&["DATE", "date"]);
    let length = meta.tags.get(&["LENGTH", "TLEN"]);

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
    layout.set_alignment(pango::Alignment::Left);
    layout.set_ellipsize(pango::EllipsizeMode::End);
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
    layout.set_alignment(pango::Alignment::Right);
    layout.set_ellipsize(pango::EllipsizeMode::Start);
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
