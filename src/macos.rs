use crate::{Browser, Error, ErrorKind, Result};
pub use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus};

/// Deal with opening of browsers on Mac OS X, using `open` command
#[inline]
pub fn open_browser_internal(browser: Browser, url: &str) -> Result<ExitStatus> {
    let mut cmd = Command::new("open");
    match browser {
        Browser::Default => cmd.arg(url).status(),
        _ => {
            let app: Option<&str> = match browser {
                Browser::Firefox => Some("Firefox"),
                Browser::Chrome => Some("Google Chrome"),
                Browser::Opera => Some("Opera"),
                Browser::Safari => Some("Safari"),
                Browser::WebPositive => Some("WebPositive"),
                _ => None,
            };
            match app {
                Some(name) => cmd.arg("-a").arg(name).arg(url).status(),
                None => Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Unsupported browser {:?}", browser),
                )),
            }
        }
    }
}
