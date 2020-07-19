use crate::{Browser, Error, ErrorKind, Result};
pub use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus};

/// Deal with opening of browsers on Android
#[inline]
pub fn open_browser_internal(_: Browser, url: &str) -> Result<ExitStatus> {
    Command::new("am")
        .arg("start")
        .arg("--user")
        .arg("0")
        .arg("-a")
        .arg("android.intent.action.VIEW")
        .arg("-d")
        .arg(url)
        .status()
}
