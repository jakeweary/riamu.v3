use std::path::PathBuf;
use std::{env, error, result};

use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
struct Callbacks;

impl ParseCallbacks for Callbacks {
  fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
    if name.starts_with("FP_") {
      return MacroParsingBehavior::Ignore;
    }
    MacroParsingBehavior::Default
  }
}

fn main() -> Result<()> {
  let out_dir = env::var("OUT_DIR")?;
  let out_dir = PathBuf::from(&out_dir);

  // let vips = pkg_config::probe_library("vips")?;
  let fc = pkg_config::probe_library("fontconfig")?;
  let rsvg = pkg_config::probe_library("librsvg-2.0")?;

  let includes = concat! {
    // "#include <vips/vips.h>\n",
    "#include <fontconfig/fontconfig.h>\n",
    "#include <librsvg/rsvg.h>\n",
  };

  bindgen::Builder::default()
    .parse_callbacks(Box::new(Callbacks))
    .clang_args(fc.include_paths.iter().map(|p| format!("-I{}", p.to_string_lossy())))
    .clang_args(rsvg.include_paths.iter().map(|p| format!("-I{}", p.to_string_lossy())))
    .header_contents("includes.h", includes)
    .generate()?
    .write_to_file(out_dir.join("bindings.rs"))?;

  Ok(())
}
