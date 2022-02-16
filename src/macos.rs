use crate::{Browser, Error, ErrorKind, Result};
use std::process::Command;

mod common;
use common::from_status;

/// Deal with opening of browsers on Mac OS X, using `open` command
#[inline]
pub fn open_browser_internal(browser: Browser, url: &str) -> Result<()> {
    let mut cmd = Command::new("open");
    match browser {
        Browser::Default => from_status(cmd.arg(url).status()),
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
                Some(name) => from_status(cmd.arg("-a").arg(name).arg(url).status()),
                None => Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Unsupported browser {:?}", browser),
                )),
            }
        }
    }
}
