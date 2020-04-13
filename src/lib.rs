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
use std::process::{ExitStatus, Output, Stdio};
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

pub struct BrowserOptions {
    pub browser: Option<Browser>,
    pub suppress_output: Option<bool>,
    pub url: String,
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
    open_browser_with_options(BrowserOptions {
        browser: Some(browser), url: url.into(), suppress_output: Some(false)
    })
}

impl BrowserOptions {
    pub fn create(url: &str) -> BrowserOptions {
        BrowserOptions {
            browser: None,
            suppress_output: None,
            url: url.into()
        }
    }

    pub fn create_with_suppressed_output(url: &str) -> BrowserOptions {
        BrowserOptions {
            browser: None,
            suppress_output: Some(true),
            url: url.into()
        }
    }
}

/// Opens the specified URL on the specific browser (if available) requested. Return semantics are
/// the same as for [open](fn.open.html).
///
/// # Examples
/// ```no_run
/// use webbrowser::{open_browser_with_options, BrowserOptions};
///
/// if open_browser_with_options(BrowserOptions::create("http://github.com")).is_ok() {
///     // ...
/// }
/// ```
pub fn open_browser_with_options(options: BrowserOptions) -> Result<Output> {
    open_browser_internal(
        options.browser.unwrap_or(Browser::default()),
        options.url.as_str(),
        options.suppress_output.unwrap_or(false)
    ).and_then(|status| {
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
fn open_browser_internal(browser: Browser, url: &str, _: bool) -> Result<ExitStatus> {
    use winapi::shared::winerror::SUCCEEDED;
    use winapi::um::combaseapi::{CoInitializeEx, CoUninitialize};
    use winapi::um::objbase::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE};
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winuser::SW_SHOWNORMAL;
    match browser {
        Browser::Default => {
            static OPEN: &[u16] = &['o' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 0x0000];
            let url =
                U16CString::from_str(url).map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
            let code = unsafe {
                let coinitializeex_result = CoInitializeEx(
                    ptr::null_mut(),
                    COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
                );
                let code = ShellExecuteW(
                    ptr::null_mut(),
                    OPEN.as_ptr(),
                    url.as_ptr(),
                    ptr::null(),
                    ptr::null(),
                    SW_SHOWNORMAL,
                ) as usize as i32;
                if SUCCEEDED(coinitializeex_result) {
                    CoUninitialize();
                }
                code
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
fn open_browser_internal(browser: Browser, url: &str, _: bool) -> Result<ExitStatus> {
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

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn adapt_command(cmd: &mut Command, suppress_output: bool) -> &mut Command {
    if suppress_output {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }

    cmd
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
fn open_browser_internal(browser: Browser, url: &str, suppress_output: bool) -> Result<ExitStatus> {
    match browser {
        Browser::Default => open_on_unix_using_browser_env(url, suppress_output)
            .or_else(|_| -> Result<ExitStatus> { adapt_command(&mut Command::new("xdg-open"), suppress_output).arg(url).status() })
            .or_else(|r| -> Result<ExitStatus> {
                if let Ok(desktop) = ::std::env::var("XDG_CURRENT_DESKTOP") {
                    if desktop == "KDE" {
                        return adapt_command(&mut Command::new("kioclient"), suppress_output).arg("exec").arg(url).status();
                    }
                }
                Err(r) // If either `if` check fails, fall through to the next or_else
            })
            .or_else(|_| -> Result<ExitStatus> { adapt_command(&mut Command::new("gvfs-open"), suppress_output).arg(url).status() })
            .or_else(|_| -> Result<ExitStatus> { adapt_command(&mut Command::new("gnome-open"), suppress_output).arg(url).status() })
            .or_else(|_| -> Result<ExitStatus> {
                adapt_command(&mut Command::new("kioclient"), suppress_output).arg("exec").arg(url).status()
            })
            .or_else(|e| -> Result<ExitStatus> {
                if let Ok(_child) = adapt_command(&mut Command::new("x-www-browser"), suppress_output).arg(url).spawn() {
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
    target_os = "openbsd"
))]
fn open_on_unix_using_browser_env(url: &str, suppress_output: bool) -> Result<ExitStatus> {
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
            if let Ok(status) = adapt_command(&mut cmd, suppress_output).status() {
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
fn test_open_with_options() {
    assert!(open_browser_with_options(BrowserOptions::create("http://github.com")).is_ok());
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
