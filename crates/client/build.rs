use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, error, result, str};

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

fn envs() -> Result<()> {
  let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
  println!("cargo:rustc-env=BUILD_TIMESTAMP={ts:?}");

  for name in ["OPT_LEVEL", "DEBUG", "PROFILE", "HOST", "TARGET", "RUSTUP_TOOLCHAIN"] {
    let value = env::var(name)?;
    println!("cargo:rustc-env=BUILD_{name}={value}");
  }

  for name in ["cargo", "rustc"] {
    let output = Command::new(name).arg("-V").output()?;
    let version = str::from_utf8(&output.stdout)?;
    let name = name.to_ascii_uppercase();
    println!("cargo:rustc-env=BUILD_{name}_VERSION={version}");
  }

  let output = Command::new("git")
    .args(["log", "-1", "--format=%ct %cs %H"])
    .output()?;
  let output = str::from_utf8(&output.stdout)?;
  let mut parts = output.split_ascii_whitespace();
  println!("cargo:rustc-env=BUILD_COMMIT_TIMESTAMP={}", parts.next().unwrap());
  println!("cargo:rustc-env=BUILD_COMMIT_DATE={}", parts.next().unwrap());
  println!("cargo:rustc-env=BUILD_COMMIT_HASH={}", parts.next().unwrap());

  Ok(())
}

fn main() -> Result<()> {
  envs()
}
