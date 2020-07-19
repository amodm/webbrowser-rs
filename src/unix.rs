use crate::{Browser, Error, ErrorKind, Result};
pub use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus};

/// Deal with opening of browsers on Linux and *BSD - currently supports only the default browser
///
/// The mechanism of opening the default browser is as follows:
/// 1. Attempt to use $BROWSER env var if available
/// 2. Attempt to open the url via xdg-open, gvfs-open, gnome-open, open, respectively, whichever works
///    first
#[inline]
pub fn open_browser_internal(browser: Browser, url: &str) -> Result<ExitStatus> {
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
