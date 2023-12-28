// https://gist.github.com/rygorous/2203834

#[rustfmt::skip]
const FP32_TO_SRGB8_TAB3: [u32; 64] = [
  0x0b0f01cb, 0x0bf401ae, 0x0ccb0195, 0x0d950180, 0x0e56016e, 0x0f0d015e, 0x0fbc0150, 0x10630143,
  0x11070264, 0x1238023e, 0x1357021d, 0x14660201, 0x156601e9, 0x165a01d3, 0x174401c0, 0x182401af,
  0x18fe0331, 0x1a9602fe, 0x1c1502d2, 0x1d7e02ad, 0x1ed4028d, 0x201a0270, 0x21520256, 0x227d0240,
  0x239f0443, 0x25c003fe, 0x27bf03c4, 0x29a10392, 0x2b6a0367, 0x2d1d0341, 0x2ebe031f, 0x304d0300,
  0x31d105b0, 0x34a80555, 0x37520507, 0x39d504c5, 0x3c37048b, 0x3e7c0458, 0x40a8042a, 0x42bd0401,
  0x44c20798, 0x488e071e, 0x4c1c06b6, 0x4f76065d, 0x52a50610, 0x55ac05cc, 0x5892058f, 0x5b590559,
  0x5e0c0a23, 0x631c0980, 0x67db08f6, 0x6c55087f, 0x70940818, 0x74a007bd, 0x787d076c, 0x7c330723,
  0x06970158, 0x07420142, 0x07e30130, 0x087b0120, 0x090b0112, 0x09940106, 0x0a1700fc, 0x0a9500f2,
];

#[rustfmt::skip]
const FP32_TO_SRGB8_TAB4: [u32; 104] = [
  0x0073000d, 0x007a000d, 0x0080000d, 0x0087000d, 0x008d000d, 0x0094000d, 0x009a000d, 0x00a1000d,
  0x00a7001a, 0x00b4001a, 0x00c1001a, 0x00ce001a, 0x00da001a, 0x00e7001a, 0x00f4001a, 0x0101001a,
  0x010e0033, 0x01280033, 0x01410033, 0x015b0033, 0x01750033, 0x018f0033, 0x01a80033, 0x01c20033,
  0x01dc0067, 0x020f0067, 0x02430067, 0x02760067, 0x02aa0067, 0x02dd0067, 0x03110067, 0x03440067,
  0x037800ce, 0x03df00ce, 0x044600ce, 0x04ad00ce, 0x051400ce, 0x057b00c5, 0x05dd00bc, 0x063b00b5,
  0x06970158, 0x07420142, 0x07e30130, 0x087b0120, 0x090b0112, 0x09940106, 0x0a1700fc, 0x0a9500f2,
  0x0b0f01cb, 0x0bf401ae, 0x0ccb0195, 0x0d950180, 0x0e56016e, 0x0f0d015e, 0x0fbc0150, 0x10630143,
  0x11070264, 0x1238023e, 0x1357021d, 0x14660201, 0x156601e9, 0x165a01d3, 0x174401c0, 0x182401af,
  0x18fe0331, 0x1a9602fe, 0x1c1502d2, 0x1d7e02ad, 0x1ed4028d, 0x201a0270, 0x21520256, 0x227d0240,
  0x239f0443, 0x25c003fe, 0x27bf03c4, 0x29a10392, 0x2b6a0367, 0x2d1d0341, 0x2ebe031f, 0x304d0300,
  0x31d105b0, 0x34a80555, 0x37520507, 0x39d504c5, 0x3c37048b, 0x3e7c0458, 0x40a8042a, 0x42bd0401,
  0x44c20798, 0x488e071e, 0x4c1c06b6, 0x4f76065d, 0x52a50610, 0x55ac05cc, 0x5892058f, 0x5b590559,
  0x5e0c0a23, 0x631c0980, 0x67db08f6, 0x6c55087f, 0x70940818, 0x74a007bd, 0x787d076c, 0x7c330723,
];

pub fn v1(x: f32) -> u8 {
  let almostone = f32::from_bits(0x3f7fffff); // 1-eps
  let lutthresh = f32::from_bits(0x3b800000); // 2^(-8)
  let linearsc = f32::from_bits(0x454c5d00);
  let float2int = f32::from_bits((127 + 23) << 23);

  // Clamp to [0, 1-eps]; these two values map to 0 and 1, respectively.
  // The tests are carefully written so that NaNs map to 0, same as in the reference
  // implementation.
  let f = clamp(x, 0.0, almostone);
  let u = f.to_bits();

  // Check which region this value falls into
  // linear region
  let u = if f < lutthresh {
    // use "magic value" to get float->int with rounding. (float2int)
    (f * linearsc + float2int).to_bits()
  }
  // non-linear region
  else {
    // Unpack bias, scale from table
    let tab = FP32_TO_SRGB8_TAB3[((u >> 20) % 64) as usize];
    let bias = (tab >> 16) << 9;
    let scale = tab & 0xffff;

    // Grab next-highest mantissa bits and perform linear interpolation
    let t = (u >> 12) & 0xff;
    (bias + (scale * t)) >> 16
  };

  u as u8
}

pub fn v2(x: f32) -> u8 {
  let almostone = f32::from_bits(0x3f7fffff); // 1-eps
  let minval = f32::from_bits((127 - 13) << 23);

  // Clamp to [2^(-13), 1-eps]; these two values map to 0 and 1, respectively.
  // The tests are carefully written so that NaNs map to 0, same as in the reference
  // implementation.
  let f = clamp(x, minval, almostone);
  let u = f.to_bits();

  // Do the table lookup and unpack bias, scale
  let tab_i = (u - minval.to_bits()) as usize >> 20;
  let tab = unsafe { *FP32_TO_SRGB8_TAB4.get_unchecked(tab_i) };
  let bias = (tab >> 16) << 9;
  let scale = tab & 0xffff;

  // Grab next-highest mantissa bits and perform linear interpolation
  let t = (u >> 12) & 0xff;
  let u = (bias + (scale * t)) >> 16;
  u as u8
}

fn clamp(x: f32, min: f32, max: f32) -> f32 {
  match x {
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    x if !(x > min) => min, // written this way to catch NaNs
    x if x > max => max,
    x => x,
  }
}
