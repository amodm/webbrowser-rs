//! Open URLs in the web browsers available on a platform.
//!
//! Inspired by the [webbrowser](https://docs.python.org/2/library/webbrowser.html) python library.
//!
//! Currently state of platform support is:
//!
//! * macos => default, as well as browsers listed under [Browser](enum.Browser.html)
//! * windows => default browser only
//! * linux or *bsd => default browser only (uses $BROWSER env var, failing back to xdg-open, gvfs-open and
//! gnome-open, in that order)
//! * android => default browser only
//! * ios => not supported right now
//!
//! Important note:
//!
//! * This library requires availability of browsers and a graphical environment during runtime
//! * `cargo test` will actually open the browser locally.
//!
//! # Examples
//!
//! ```no_run
//! use webbrowser;
//!
//! if webbrowser::open("http://github.com").is_ok() {
//!     // ...
//! }
//! ```

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
use android::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::*;

use std::default::Default;
use std::io::{Error, ErrorKind, Result};
use std::process::{ExitStatus, Output};
use std::str::FromStr;
use std::{error, fmt};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[cfg(target_arch = "wasm32")]
use web_sys::Window;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
/// Browser types available
pub enum Browser {
    ///Operating system's default browser
    Default,

    ///Mozilla Firefox
    Firefox,

    ///Microsoft's Internet Explorer
    InternetExplorer,

    ///Google Chrome
    Chrome,

    ///Opera
    Opera,

    ///Mac OS Safari
    Safari,

    ///Haiku's WebPositive
    WebPositive,
}

///The Error type for parsing a string into a Browser.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct ParseBrowserError;

impl fmt::Display for ParseBrowserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Invalid browser given")
    }
}

impl error::Error for ParseBrowserError {
    fn description(&self) -> &str {
        "invalid browser"
    }
}

impl Default for Browser {
    fn default() -> Self {
        Browser::Default
    }
}

impl fmt::Display for Browser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Browser::Default => f.write_str("Default"),
            Browser::Firefox => f.write_str("Firefox"),
            Browser::InternetExplorer => f.write_str("Internet Explorer"),
            Browser::Chrome => f.write_str("Chrome"),
            Browser::Opera => f.write_str("Opera"),
            Browser::Safari => f.write_str("Safari"),
            Browser::WebPositive => f.write_str("WebPositive"),
        }
    }
}

impl FromStr for Browser {
    type Err = ParseBrowserError;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "firefox" => Ok(Browser::Firefox),
            "default" => Ok(Browser::Default),
            "ie" | "internet explorer" | "internetexplorer" => Ok(Browser::InternetExplorer),
            "chrome" => Ok(Browser::Chrome),
            "opera" => Ok(Browser::Opera),
            "safari" => Ok(Browser::Safari),
            "webpositive" => Ok(Browser::WebPositive),
            _ => Err(ParseBrowserError),
        }
    }
}

/// Opens the URL on the default browser of this platform
///
/// Returns Ok(..) so long as the browser invocation was successful. An Err(..) is returned only if
/// there was an error in running the command, or if the browser was not found.
///
/// Equivalent to:
/// ```no_run
/// # use webbrowser::{Browser, open_browser};
/// # let url = "http://example.com";
/// open_browser(Browser::Default, url);
/// ```
///
/// # Examples
/// ```no_run
/// use webbrowser;
///
/// if webbrowser::open("http://github.com").is_ok() {
///     // ...
/// }
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub fn open(url: &str) -> Result<Output> {
    open_browser(Browser::Default, url)
}

#[cfg(target_arch = "wasm32")]
pub fn open(url: &str) -> Result<()> {
    let window = web_sys::window();
    match window {
        Some(w) => {
            w.open_with_url(url);
            Ok(())
        }
        None => Err(std::io::Error::new(
            ErrorKind::Other,
            "should have a window in this context",
        )),
    }
}

/// Opens the specified URL on the specific browser (if available) requested. Return semantics are
/// the same as for [open](fn.open.html).
///
/// # Examples
/// ```no_run
/// use webbrowser::{open_browser, Browser};
///
/// if open_browser(Browser::Firefox, "http://github.com").is_ok() {
///     // ...
/// }
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub fn open_browser(browser: Browser, url: &str) -> Result<Output> {
    open_browser_internal(browser, url).and_then(|status| {
        if let Some(code) = status.code() {
            if code == 0 {
                Ok(Output {
                    status: ExitStatus::from_raw(0),
                    stdout: vec![],
                    stderr: vec![],
                })
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    format!("return code {}", code),
                ))
            }
        } else {
            Err(Error::new(ErrorKind::Other, "interrupted by signal"))
        }
    })
}

/// Deal with opening of browsers on Linux and *BSD - currently supports only the default browser
///
/// The mechanism of opening the default browser is as follows:
/// 1. Attempt to use $BROWSER env var if available
/// 2. Attempt to open the url via xdg-open, gvfs-open, gnome-open, open, respectively, whichever works
///    first
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "haiku"
))]
#[inline]
fn open_browser_internal(browser: Browser, url: &str) -> Result<ExitStatus> {
    match browser {
        Browser::Default => open_on_unix_using_browser_env(url)
            .or_else(|_| -> Result<ExitStatus> { Command::new("xdg-open").arg(url).status() })
            .or_else(|r| -> Result<ExitStatus> {
                if let Ok(desktop) = ::std::env::var("XDG_CURRENT_DESKTOP") {
                    if desktop == "KDE" {
                        return Command::new("kioclient").arg("exec").arg(url).status();
                    }
                }
                Err(r) // If either `if` check fails, fall through to the next or_else
            })
            .or_else(|_| -> Result<ExitStatus> { Command::new("gvfs-open").arg(url).status() })
            .or_else(|_| -> Result<ExitStatus> { Command::new("gnome-open").arg(url).status() })
            .or_else(|_| -> Result<ExitStatus> { Command::new("open").arg(url).status() })
            .or_else(|_| -> Result<ExitStatus> {
                Command::new("kioclient").arg("exec").arg(url).status()
            })
            .or_else(|e| -> Result<ExitStatus> {
                if let Ok(_child) = Command::new("x-www-browser").arg(url).spawn() {
                    return Ok(ExitStatusExt::from_raw(0));
                }
                Err(e)
            }),
        _ => Err(Error::new(
            ErrorKind::NotFound,
            "Only the default browser is supported on this platform right now",
        )),
    }
}
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "haiku"
))]
fn open_on_unix_using_browser_env(url: &str) -> Result<ExitStatus> {
    let browsers = ::std::env::var("BROWSER")
        .map_err(|_| -> Error { Error::new(ErrorKind::NotFound, "BROWSER env not set") })?;
    for browser in browsers.split(':') {
        // $BROWSER can contain ':' delimited options, each representing a potential browser command line
        if !browser.is_empty() {
            // each browser command can have %s to represent URL, while %c needs to be replaced
            // with ':' and %% with '%'
            let cmdline = browser
                .replace("%s", url)
                .replace("%c", ":")
                .replace("%%", "%");
            let cmdarr: Vec<&str> = cmdline.split_whitespace().collect();
            let mut cmd = Command::new(&cmdarr[0]);
            if cmdarr.len() > 1 {
                cmd.args(&cmdarr[1..cmdarr.len()]);
            }
            if !browser.contains("%s") {
                // append the url as an argument only if it was not already set via %s
                cmd.arg(url);
            }
            if let Ok(status) = cmd.status() {
                return Ok(status);
            }
        }
    }
    Err(Error::new(
        ErrorKind::NotFound,
        "No valid command in $BROWSER",
    ))
}

#[cfg(not(any(
    target_os = "android",
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "haiku",
    target_arch = "wasm32"
)))]
compile_error!("Only Windows, Mac OS, Linux, *BSD and Haiku and Wasm32 are currently supported");

#[test]
fn test_open_default() {
    assert!(open("http://github.com").is_ok());
    assert!(open("http://github.com?dummy_query1=0&dummy_query2=ｎｏｎａｓｃｉｉ").is_ok());
}

#[test]
#[ignore]
fn test_open_firefox() {
    assert!(open_browser(Browser::Firefox, "http://github.com").is_ok());
}

#[test]
#[ignore]
fn test_open_chrome() {
    assert!(open_browser(Browser::Chrome, "http://github.com").is_ok());
}

#[test]
#[cfg(target_arch = "wasm32")]
fn test_open_default_wasm() {
    assert!(open("http://github.com").is_ok());
}

#[test]
#[ignore]
fn test_open_safari() {
    assert!(open_browser(Browser::Safari, "http://github.com").is_ok());
}

#[test]
#[ignore]
fn test_open_webpositive() {
    assert!(open_browser(Browser::WebPositive, "http://github.com").is_ok());
}
