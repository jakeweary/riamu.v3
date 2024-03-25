// https://microsoft.github.io/DirectX-Specs/d3d/archive/D3D11_3_FunctionalSpec.htm#UNORMtoFLOAT
// https://microsoft.github.io/DirectX-Specs/d3d/archive/D3D11_3_FunctionalSpec.htm#FLOATtoUNORM

macro_rules! impl_inner(($ident:ident, $u:ident, $f:ident) => {
  pub fn $ident(x: $f) -> $u {
    (0.5 + x * $u::MAX as $f) as $u
  }
});

macro_rules! impl_outer(($f:ident) => {
  pub mod $f {
    impl_inner!(unorm8, u8, $f);
    impl_inner!(unorm16, u16, $f);
  }
});

impl_outer!(f32);
impl_outer!(f64);

macro_rules! impl_inner(($u:ident, $f:ident) => {
  pub fn $f(x: $u) -> $f {
    x as $f / $u::MAX as $f
  }
});

macro_rules! impl_outer(($ident:ident, $u:ident) => {
  pub mod $ident {
    impl_inner!($u, f32);
    impl_inner!($u, f64);
  }
});

impl_outer!(unorm8, u8);
impl_outer!(unorm16, u16);
