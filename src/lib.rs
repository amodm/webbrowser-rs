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
//! * linux => default browser only (uses $BROWSER env var, failing back to xdg-open, gvfs-open and
//! gnome-open, in that order)
//! * android => not supported right now
//! * ios => not supported right now
//!
//! Important note:
//!
//! * This library requires availability of browsers and a graphical environment during runtime
//! * `cargo test` will actually open the browser locally

use std::process::{Command, Output};
use std::io::{Result, Error, ErrorKind};
use std::env;

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
    open_browser_internal(browser, url)
}

/// Deal with opening of browsers on Windows, using `start` command
#[cfg(target_os = "windows")]
#[inline]
fn open_browser_internal(browser: Browser, url: &str) -> Result<Output> {
    match browser {
        Browser::Default => Command::new("cmd").arg("/C").arg("start").arg(url).output(),
        _ => Err(Error::new(
            ErrorKind::NotFound,
            "Only the default browser is supported on this platform right now"
        ))
    }
}

/// Deal with opening of browsers on Mac OS X, using `open` command
#[cfg(target_os = "macos")]
#[inline]
fn open_browser_internal(browser: Browser, url: &str) -> Result<Output> {
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

/// Deal with opening of browsers on Linux - currently supports only the default browser
///
/// The mechanism of opening the default browser is as follows:
/// 1. Attempt to use $BROWSER env var if available
/// 2. Attempt to open the url via xdg-open, gvfs-open, gnome-open, respectively, whichever works
///    first
#[cfg(target_os = "linux")]
#[inline]
fn open_browser_internal(browser: Browser, url: &str) -> Result<Output> {
    match browser {
        Browser::Default => open_on_linux_using_browser_env(url)
            .or_else(|_| -> Result<Output> {Command::new("xdg-open").arg(url).output()})
            .or_else(|_| -> Result<Output> {Command::new("gvfs-open").arg(url).output()})
            .or_else(|_| -> Result<Output> {Command::new("gnome-open").arg(url).output()}),
        _ => Err(Error::new(
                ErrorKind::NotFound,
                "Only the default browser is supported on this platform right now"
            ))
    }
}

/// Open on Linux using the $BROWSER env var
#[cfg(target_os = "linux")]
fn open_on_linux_using_browser_env(url: &str) -> Result<Output> {
    let browsers = env::var("BROWSER").map_err(|_| -> Error { Error::new(ErrorKind::NotFound, format!("BROWSER env not set")) })?;
    for browser in browsers.split(':') { // $BROWSER can contain ':' delimited options, each representing a potential browser command line
        if !browser.is_empty() {
            // each browser command can have %s to represent URL, while %c needs to be replaced
            // with ':' and %% with '%'
            let cmdline = browser.replace("%s", url).replace("%c", ":").replace("%%", "%");
            let cmdarr: Vec<&str> = cmdline.split_whitespace().collect();
            let mut cmd = Command::new(&cmdarr[0]);
            if cmdarr.len() > 1 {
                cmd.args(&cmdarr[1..cmdarr.len()]);
            }
            if !browser.contains("%s") {
                // append the url as an argument only if it was not already set via %s
                cmd.arg(url);
            }
            if let Ok(output) = cmd.output() {
                return Ok(output);
            }
        }
    }
    return Err(Error::new(ErrorKind::NotFound, "No valid command in $BROWSER"));
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
compile_error!("Only Windows, Mac OS and Linux are currently supported");

#[test]
fn test_open_default() {
    assert!(open("http://github.com").is_ok());
}

#[test]
#[cfg(target_os = "macos")]
fn test_open_browser() {
    assert!(open_browser(Browser::Firefox, "http://github.com").is_ok());
}
