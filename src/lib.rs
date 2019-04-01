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
//! * android => not supported right now
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
extern crate widestring;
#[cfg(windows)]
extern crate winapi;

use std::default::Default;
use std::io::{Error, ErrorKind, Result};
use std::process::{ExitStatus, Output};
use std::str::FromStr;
use std::{error, fmt};

#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
#[cfg(windows)]
use std::ptr;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[cfg(not(windows))]
use std::process::Command;

#[cfg(windows)]
use widestring::U16CString;

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
pub fn open(url: &str) -> Result<Output> {
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

/// Deal with opening of browsers on Windows, using [`ShellExecuteW`](
/// https://docs.microsoft.com/en-us/windows/desktop/api/shellapi/nf-shellapi-shellexecutew)
/// fucntion.
#[cfg(target_os = "windows")]
#[inline]
fn open_browser_internal(browser: Browser, url: &str) -> Result<ExitStatus> {
    use winapi::um::combaseapi::CoInitializeEx;
    use winapi::um::objbase::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE};
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winuser::SW_SHOWNORMAL;

    match browser {
        Browser::Default => {
            static OPEN: &[u16] = &['o' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 0x0000];
            let url =
                U16CString::from_str(url).map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
            let code = unsafe {
                let _ = CoInitializeEx(
                    ptr::null_mut(),
                    COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
                );
                ShellExecuteW(
                    ptr::null_mut(),
                    OPEN.as_ptr(),
                    url.as_ptr(),
                    ptr::null(),
                    ptr::null(),
                    SW_SHOWNORMAL,
                ) as usize as i32
            };
            if code > 32 {
                Ok(ExitStatus::from_raw(0))
            } else {
                Err(Error::last_os_error())
            }
        }
        _ => Err(Error::new(
            ErrorKind::NotFound,
            "Only the default browser is supported on this platform right now",
        )),
    }
}

/// Deal with opening of browsers on Mac OS X, using `open` command
#[cfg(target_os = "macos")]
#[inline]
fn open_browser_internal(browser: Browser, url: &str) -> Result<ExitStatus> {
    let mut cmd = Command::new("open");
    match browser {
        Browser::Default => cmd.arg(url).status(),
        _ => {
            let app: Option<&str> = match browser {
                Browser::Firefox => Some("Firefox"),
                Browser::Chrome => Some("Google Chrome"),
                Browser::Opera => Some("Opera"),
                Browser::Safari => Some("Safari"),
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

/// Deal with opening of browsers on Linux and *BSD - currently supports only the default browser
///
/// The mechanism of opening the default browser is as follows:
/// 1. Attempt to use $BROWSER env var if available
/// 2. Attempt to open the url via xdg-open, gvfs-open, gnome-open, respectively, whichever works
///    first
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
#[inline]
fn open_browser_internal(browser: Browser, url: &str) -> Result<ExitStatus> {
    match browser {
        Browser::Default => open_on_unix_using_browser_env(url)
            .or_else(|_| -> Result<ExitStatus> { Command::new("xdg-open").arg(url).status() })
            .or_else(|_| -> Result<ExitStatus> { Command::new("gvfs-open").arg(url).status() })
            .or_else(|_| -> Result<ExitStatus> { Command::new("gnome-open").arg(url).status() }),
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
    target_os = "openbsd"
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
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
compile_error!("Only Windows, Mac OS, Linux and *BSD are currently supported");

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
#[ignore]
#[cfg(target_os = "windows")]
fn test_open_internet_explorer() {
    assert!(open_browser(Browser::InternetExplorer, "http://github.com").is_ok());
}

#[test]
#[ignore]
fn test_open_safari() {
    assert!(open_browser(Browser::Safari, "http://github.com").is_ok());
}
