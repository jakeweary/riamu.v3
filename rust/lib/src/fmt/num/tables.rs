// https://en.wikipedia.org/wiki/Unit_prefix
// https://en.wikipedia.org/wiki/Metric_prefix
// https://en.wikipedia.org/wiki/Binary_prefix

use super::*;

#[rustfmt::skip]
pub static SI: [Si; 20] = [
  Si { short: 'q', long: "quecto" },
  Si { short: 'r', long:  "ronto" },
  Si { short: 'y', long:  "yocto" },
  Si { short: 'z', long:  "zepto" },
  Si { short: 'a', long:   "atto" },
  Si { short: 'f', long:  "femto" },
  Si { short: 'p', long:   "pico" },
  Si { short: 'n', long:   "nano" },
  Si { short: 'μ', long:  "micro" },
  Si { short: 'm', long:  "milli" },
  Si { short: 'k', long:   "kilo" },
  Si { short: 'M', long:   "mega" },
  Si { short: 'G', long:   "giga" },
  Si { short: 'T', long:   "tera" },
  Si { short: 'P', long:   "peta" },
  Si { short: 'E', long:    "exa" },
  Si { short: 'Z', long:  "zetta" },
  Si { short: 'Y', long:  "yotta" },
  Si { short: 'R', long:  "ronna" },
  Si { short: 'Q', long: "quetta" },
];

#[rustfmt::skip]
pub static IEC: [Iec; 10] = [
  Iec { short: 'K', long:  "kibi" },
  Iec { short: 'M', long:  "mebi" },
  Iec { short: 'G', long:  "gibi" },
  Iec { short: 'T', long:  "tebi" },
  Iec { short: 'P', long:  "pebi" },
  Iec { short: 'E', long:  "exbi" },
  Iec { short: 'Z', long:  "zebi" },
  Iec { short: 'Y', long:  "yobi" },
  Iec { short: 'R', long:  "robi" }, // as of 2022, suggested but not yet adopted
  Iec { short: 'Q', long: "quebi" }, // https://doi.org/10.1088%2F1681-7575%2Fac6afd
];
