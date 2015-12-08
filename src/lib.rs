//! Open URLs in the web browsers available on a platform
//!
//! Inspired by the [webbrowser](https://docs.python.org/2/library/webbrowser.html) python library
//!
//! #Examples
//!
//! ```
//! use webbrowser;
//! 
//! if webbrowser::open("http://github.com").is_ok() {
//!     // ...
//! }
//! ```
//!
//! Currently state of platform support is:
//!
//! * macos => default, as well as browsers listed under [Browser](enum.Browser.html)
//! * windows => default browser only
//! * linux => default browser only
//! * android => not supported right now
//! * ios => not supported right now
//!
//! Important note:
//!
//! * This library requires availability of browsers and a graphical environment during runtime
//! * `cargo test` will actually open the browser locally

use std::process::{Command, Output};
use std::io::{Result, Error, ErrorKind};

#[derive(Debug)]
/// Browser types available
pub enum Browser {
    Default,
    Firefox,
    InternetExplorer,
    Chrome,
    Opera,
    Safari
}

/// Opens the URL on the default browser of this platform
///
/// Returns Ok(..) so long as the browser invocation was successful. An Err(..) is returned only if
/// there was an error in running the command, or if the browser was not found
///
/// # Examples
/// ```
/// use webbrowser;
///
/// if webbrowser::open("http://github.com").is_ok() {
///     // ...
/// }
/// ```
pub fn open(url: &str) -> Result<Output> {
    open_browser(Browser::Default, url)
}

/// Opens the specified URL on the specific browser (if available) requested. Return semantics are
/// the same as for [open](fn.open.html)
///
/// # Examples
/// ```
/// use webbrowser::{open_browser, Browser};
///
/// if open_browser(Browser::Firefox, "http://github.com").is_ok() {
///     // ...
/// }
/// ```
pub fn open_browser(browser: Browser, url: &str) -> Result<Output> {
    let os = std::env::consts::OS;
    match os {
        "macos" => open_on_macos(browser, url),
        "windows" => open_on_windows(browser, url),
        "linux" => open_on_linux(browser, url),
        _ => Err(Error::new(ErrorKind::NotFound, format!("Platform {} not yet supported by this library", os)))
    }
}

/// Deal with opening of browsers on Mac OS X, using `open` command
fn open_on_macos(browser: Browser, url: &str) -> Result<Output> {
    let mut cmd = Command::new("open");
    match browser {
        Browser::Default => cmd.arg(url).output(),
        _ => {
            let app: Option<&str> = match browser {
                Browser::Firefox => Some("Firefox"),
                Browser::Chrome => Some("Google Chrome"),
                Browser::Opera => Some("Opera"),
                Browser::Safari => Some("Safari"),
                _ => None
            };
            match app {
                Some(name) => cmd.arg("-a").arg(name).arg(url).output(),
                None => Err(Error::new(ErrorKind::NotFound, format!("Unsupported browser {:?}", browser)))
            }
        }
    }
}

/// Deal with opening of browsers on Windows, using `start link` command
fn open_on_windows(browser: Browser, url: &str) -> Result<Output> {
    match browser {
        Browser::Default => Command::new("start").arg("link").arg(url).output(),
        _ => Err(Error::new(
                ErrorKind::NotFound,
                "Only the default browser is supported on this platform right now"
            ))
    }
}

/// Deal with opening of browsers on Linux, using `xdg-open` command
fn open_on_linux(browser: Browser, url: &str) -> Result<Output> {
    match browser {
        Browser::Default => Command::new("xdg-open").arg(url).output(),
        _ => Err(Error::new(
                ErrorKind::NotFound,
                "Only the default browser is supported on this platform right now"
            ))
    }
}

#[test]
fn test_open_default() {
    assert!(open("http://github.com").is_ok());
}

#[test]
fn test_open_browser() {
    assert!(open_browser(Browser::Firefox, "http://github.com").is_ok());
}
