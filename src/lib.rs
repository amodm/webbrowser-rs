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
#[cfg(target_os = "macos")]
extern crate url;
#[cfg(target_os = "macos")]
extern crate launch_services;
#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
extern crate core_foundation_sys;

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

#[cfg(all(not(target_os = "macos"), not(windows)))]
use std::process::Command;

#[cfg(windows)]
use widestring::U16CString;

#[cfg(target_os = "macos")]
use launch_services::{
    open_url,
    open_from_url_spec,
    application_urls_for_bundle_identifier,
    LSLaunchFlags,
    LSLaunchURLSpec,
};

#[cfg(target_os = "macos")]
use core_foundation_sys::base::{kCFAllocatorDefault, CFAllocatorRef};

#[cfg(target_os = "macos")]
use core_foundation::{
    base::TCFType,
    url::{CFURL, CFURLRef},
    string::{CFString, CFStringRef},
    array::CFArray,
};

#[cfg(target_os = "macos")]
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
use url::Url;

#[cfg(target_os = "macos")]
#[link(name = "CoreServices", kind = "framework")]
extern "C" {
    fn CFURLCreateWithString(
        allocator: CFAllocatorRef,
        urlString: CFStringRef,
        baseURL: CFURLRef,
    ) -> CFURLRef;
}


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

#[cfg(target_os = "macos")]
#[inline]
fn sanitize_url(value: &str) -> Option<String> {
    Some(Url::parse(value).ok()?.into_string())
}

#[cfg(target_os = "macos")]
#[inline]
fn str_to_url(value: &str) -> Option<CFURL> {
    let url = CFString::new(&sanitize_url(value)?);

    let ptr = unsafe {
        CFURLCreateWithString(
            kCFAllocatorDefault,
            url.as_concrete_TypeRef(),
            std::ptr::null(),
        )
    };

    if ptr.is_null() {
        None
    } else {
        Some(unsafe { TCFType::wrap_under_create_rule(ptr) })
    }
}

#[cfg(target_os = "macos")]
#[inline]
fn mopen(value: &str) -> Result<Option<PathBuf>> {
    if let Some(url) = str_to_url(value) {
        match open_url(&url) {
            Ok(path) => Ok(path.to_path()),
            Err(code) => Err(Error::new(
                ErrorKind::Other,
                format!("return code {}", code),
            )),
        }
    } else {
        Err(Error::new(ErrorKind::Other, "Provided url is not openable"))
    }
}

#[cfg(target_os = "macos")]
#[inline]
pub fn apps_for_bundle_id(bundle_id: &str) -> Option<Vec<PathBuf>> {
    let bundle_id = CFString::new(bundle_id);
    match application_urls_for_bundle_identifier(&bundle_id) {
        Ok(apps) => Some(apps.iter().filter_map(|v| v.to_path()).collect()),
        Err(_) => None,
    }
}

#[cfg(target_os = "macos")]
#[inline]
pub fn app_for_bundle_id(bundle_id: &str) -> Option<PathBuf> {
    let mut apps = apps_for_bundle_id(bundle_id)?;
    if apps.is_empty() {
        None
    } else {
        Some(apps.remove(0))
    }
}

#[cfg(target_os = "macos")]
#[inline]
fn remap_app(app: &Path) -> Result<CFURL> {
    match CFURL::from_path(app, true) {
        None => Err(Error::new(
            ErrorKind::Other,
            "Provided app url is not valid",
        )),
        Some(res) => Ok(res),
    }
}

#[cfg(target_os = "macos")]
#[inline]
fn remap_url(value: &str) -> Result<CFArray<CFURL>> {
    let mut res: Vec<CFURL> = Vec::new();
    match str_to_url(value) {
        None => return Err(Error::new(ErrorKind::Other, "Provided urls are not valid")),
        Some(url) => {
            res.push(url);
        }
    }

    Ok(CFArray::<CFURL>::from_CFTypes(&res[..]))
}

#[cfg(target_os = "macos")]
#[inline]
fn mopen_complex(app: &Path, value: &str) -> Result<Option<PathBuf>> {
    let spec = LSLaunchURLSpec {
        app: Some(remap_app(app)?),
        urls: Some(remap_url(value)?),
        flags: LSLaunchFlags::DEFAULTS | LSLaunchFlags::ASYNC,
        ..Default::default()
    };

    match open_from_url_spec(spec) {
        Ok(path) => Ok(path.to_path()),
        Err(code) => Err(Error::new(
            ErrorKind::Other,
            format!("return code {}", code),
        )),
    }
}

#[cfg(target_os = "macos")]
#[inline]
fn transform_result(result: Result<Option<PathBuf>>) -> Result<ExitStatus> {
    match result {
        Ok(_) => Ok(ExitStatus::from_raw(0)),
        Err(err) => {
            println!("{:#?}", err);
            Err(err)
        },
    }
}

/// Deal with opening of browsers on Mac OS X, using Core Services framework
#[cfg(target_os = "macos")]
#[inline]
fn open_browser_internal(browser: Browser, url: &str) -> Result<ExitStatus> {
    match browser {
        Browser::Default => transform_result(mopen(url)),
        _ => {
            let app: Option<&str> = match browser {
                Browser::Firefox => Some("org.mozilla.firefox"),
                Browser::Chrome => Some("com.google.chrome"),
                Browser::Opera => Some("com.operasoftware.Opera"),
                Browser::Safari => Some("com.apple.Safari"),
                _ => None,
            };
            match app {
                Some(bundle_id) => {
                    if let Some(browser_path) = app_for_bundle_id(bundle_id) {
                        transform_result(mopen_complex(&browser_path, url))
                    } else {
                        Err(Error::new(
                            ErrorKind::NotFound,
                            format!("Not installed browser {:?}", browser)
                        ))
                    }
                },
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
