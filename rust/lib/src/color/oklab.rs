// https://bottosson.github.io/posts/
// https://bottosson.github.io/misc/colorpicker/
// https://bottosson.github.io/misc/ok_color.h

// https://raphlinus.github.io/color/2021/01/18/oklab-critique
// https://github.com/svgeesus/svgeesus.github.io/blob/master/Color/OKLab-notes.md

#![allow(non_snake_case)]

use self::helpers::*;

pub use self::hsl::*;
pub use self::hsv::*;
pub use self::lab::*;

mod hsl;
mod hsv;
mod lab;

pub mod gamut_clip;
pub mod helpers;
