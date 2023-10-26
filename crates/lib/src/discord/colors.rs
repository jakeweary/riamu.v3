pub type Color = u32;

pub struct ColorPair {
  pub light: Color,
  pub dark: Color,
}

pub static OK: &ColorPair = &TABLE[1];
pub static INFO: &ColorPair = &TABLE[2];
pub static WARN: &ColorPair = &TABLE[5];
pub static ERROR: &ColorPair = &TABLE[7];

#[rustfmt::skip]
pub static TABLE: [ColorPair; 10] = [
  ColorPair { light: 0x1abc9c, dark: 0x11806a },
  ColorPair { light: 0x2ecc71, dark: 0x1f8b4c },
  ColorPair { light: 0x3498db, dark: 0x206694 },
  ColorPair { light: 0x9b59b6, dark: 0x71368a },
  ColorPair { light: 0xe91e63, dark: 0xad1457 },
  ColorPair { light: 0xf1c40f, dark: 0xc27c0e },
  ColorPair { light: 0xe67e22, dark: 0xa84300 },
  ColorPair { light: 0xe74c3c, dark: 0x992d22 },
  ColorPair { light: 0x95a5a6, dark: 0x979c9f },
  ColorPair { light: 0x607d8b, dark: 0x546e7a },
];
