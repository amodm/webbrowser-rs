use crate::{Browser, Error, ErrorKind, Result};
use std::ffi::OsStr;
pub use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus};

/// Deal with opening of browsers on Android
#[inline]
pub fn open_browser_internal<P: AsRef<OsStr>>(_: Browser, url: P) -> Result<ExitStatus> {
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
