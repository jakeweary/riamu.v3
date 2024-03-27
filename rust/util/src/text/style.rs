use std::mem;

fn cc(cc: u32) -> char {
  unsafe { mem::transmute(cc) }
}

pub fn fullwidth_cjk(c: char) -> char {
  match c {
    '!'..='~' => cc(0xfee0 + c as u32),
    ' ' => '\u{3000}',
    _ => c,
  }
}

pub fn regional_indicators(c: char) -> char {
  match c {
    'A'..='Z' | 'a'..='z' => cc(0x1f1e5 + c as u32 % 32),
    _ => c,
  }
}

pub fn monospace(c: char) -> char {
  match c {
    'A'..='Z' => cc(0x1d66f + c as u32 % 32),
    'a'..='z' => cc(0x1d689 + c as u32 % 32),
    '0'..='9' => cc(0x1d7f6 + c as u32 % 16),
    _ => c,
  }
}

pub fn double_struck(c: char) -> char {
  match c {
    'A'..='Z' => cc(0x1d537 + c as u32 % 32),
    'a'..='z' => cc(0x1d551 + c as u32 % 32),
    '0'..='9' => cc(0x1d7d8 + c as u32 % 16),
    _ => c,
  }
}

pub mod fractur {
  use super::cc;

  pub fn regular(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d503 + c as u32 % 32),
      'a'..='z' => cc(0x1d51d + c as u32 % 32),
      _ => c,
    }
  }

  pub fn bold(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d56b + c as u32 % 32),
      'a'..='z' => cc(0x1d585 + c as u32 % 32),
      _ => c,
    }
  }
}

pub mod script {
  use super::cc;

  pub fn regular(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d49b + c as u32 % 32),
      'a'..='z' => cc(0x1d4b5 + c as u32 % 32),
      _ => c,
    }
  }

  pub fn bold(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d4cf + c as u32 % 32),
      'a'..='z' => cc(0x1d4e9 + c as u32 % 32),
      _ => c,
    }
  }
}

pub mod serif {
  use super::cc;

  pub fn bold(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d3ff + c as u32 % 32),
      'a'..='z' => cc(0x1d419 + c as u32 % 32),
      '0'..='9' => cc(0x1d7ce + c as u32 % 16),
      _ => c,
    }
  }

  pub fn italic(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d433 + c as u32 % 32),
      'a'..='z' => cc(0x1d44d + c as u32 % 32),
      _ => c,
    }
  }

  pub fn bold_italic(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d467 + c as u32 % 32),
      'a'..='z' => cc(0x1d481 + c as u32 % 32),
      _ => c,
    }
  }
}

pub mod sans_serif {
  use super::cc;

  pub fn regular(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d59f + c as u32 % 32),
      'a'..='z' => cc(0x1d5b9 + c as u32 % 32),
      '0'..='9' => cc(0x1d7e2 + c as u32 % 16),
      _ => c,
    }
  }

  pub fn bold(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d5d3 + c as u32 % 32),
      'a'..='z' => cc(0x1d5ed + c as u32 % 32),
      '0'..='9' => cc(0x1d7ec + c as u32 % 16),
      _ => c,
    }
  }

  pub fn italic(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d607 + c as u32 % 32),
      'a'..='z' => cc(0x1d621 + c as u32 % 32),
      _ => c,
    }
  }

  pub fn bold_italic(c: char) -> char {
    match c {
      'A'..='Z' => cc(0x1d63b + c as u32 % 32),
      'a'..='z' => cc(0x1d655 + c as u32 % 32),
      _ => c,
    }
  }
}
