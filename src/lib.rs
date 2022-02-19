//! Open URLs in the web browsers available on a platform.
//!
//! Inspired by the [webbrowser](https://docs.python.org/2/library/webbrowser.html) python library.
//!
//! ## Examples
//!
//! ```no_run
//! use webbrowser;
//!
//! if webbrowser::open("http://github.com").is_ok() {
//!     // ...
//! }
//! ```
//!
//! ## Platform Support Status
//!
//! | Platform | Supported | Browsers | Test status |
//! |----------|-----------|----------|-------------|
//! | macos    | ✅        | default + [others](https://docs.rs/webbrowser/latest/webbrowser/enum.Browser.html) | ✅ |
//! | windows  | ✅        | default only | ✅ |
//! | linux/*bsd  | ✅     | default only (respects $BROWSER env var, so can be used with other browsers) | ✅ |
//! | android  | ✅        | default only | ✅ |
//! | wasm     | ✅        | default only | ✅ |
//! | haiku    | ✅ (experimental) | default only | ❌ |
//! | ios      | ❌        | unsupported | ❌ |
//!
//! ## Consistent Behaviour
//! `webbrowser` defines consistent behaviour on all platforms as follows:
//! * **Non-Blocking** for GUI based browsers (e.g. Firefox, Chrome etc.), while **Blocking** for text based browser (e.g. lynx etc.)
//! * **Suppressed output** by default for GUI based browsers, so that their stdout/stderr don't pollute the main program's output. This can be overridden by `webbrowser::open_browser_with_options`.

#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(target_os = "android", path = "android.rs")]
#[cfg_attr(target_arch = "wasm32", path = "wasm.rs")]
#[cfg_attr(windows, path = "windows.rs")]
#[cfg_attr(
    any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "haiku"
    ),
    path = "unix.rs"
)]
mod os;

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

use std::default::Default;
use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;
use std::{error, fmt};

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

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
/// BrowserOptions to override certain default behaviour
///
/// e.g. by default, we suppress stdout/stderr, but that behaviour can be overridden here
pub struct BrowserOptions {
    pub suppress_output: bool,
}

impl fmt::Display for BrowserOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "BrowserOptions(supress_output={})",
            self.suppress_output
        ))
    }
}

impl std::default::Default for BrowserOptions {
    fn default() -> Self {
        BrowserOptions {
            suppress_output: true,
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
pub fn open(url: &str) -> Result<()> {
    open_browser(Browser::Default, url)
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
pub fn open_browser(browser: Browser, url: &str) -> Result<()> {
    open_browser_with_options(browser, url, &BrowserOptions::default())
}

/// Opens the specified URL on the specific browser (if available) requested, while overriding the
/// default options.
///
/// Return semantics are
/// the same as for [open](fn.open.html).
///
/// # Examples
/// ```no_run
/// use webbrowser::{open_browser_with_options, Browser, BrowserOptions};
///
/// if open_browser_with_options(Browser::Default, "http://github.com", &BrowserOptions { suppress_output: false }).is_ok() {
///     // ...
/// }
/// ```
pub fn open_browser_with_options(
    browser: Browser,
    url: &str,
    options: &BrowserOptions,
) -> Result<()> {
    let url_s: String = match url::Url::parse(url) {
        Ok(u) => u.as_str().into(),
        Err(_) => url.into(),
    };
    os::open_browser_internal(browser, &url_s, options)
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
#[ignore]
fn test_open_safari() {
    assert!(open_browser(Browser::Safari, "http://github.com").is_ok());
}

#[test]
#[ignore]
fn test_open_webpositive() {
    assert!(open_browser(Browser::WebPositive, "http://github.com").is_ok());
}
