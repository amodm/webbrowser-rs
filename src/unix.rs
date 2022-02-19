use crate::{Browser, Error, ErrorKind, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

macro_rules! try_browser {
    ( $name:expr, $( $arg:expr ),+ ) => {
        for_matching_path($name, |pb| {
            let mut cmd = Command::new(pb);
            $(
                cmd.arg($arg);
            )+
            run_command(&mut cmd, !is_text_browser(&pb))
        })
    }
}

/// Deal with opening of browsers on Linux and *BSD - currently supports only the default browser
///
/// The mechanism of opening the default browser is as follows:
/// 1. Attempt to use $BROWSER env var if available
/// 2. Attempt to use xdg-open
/// 3. Attempt to use window manager specific commands, like gnome-open, kde-open etc.
/// 4. Fallback to x-www-browser
#[inline]
pub fn open_browser_internal(_: Browser, url: &str) -> Result<()> {
    // we first try with the $BROWSER env
    try_with_browser_env(url)
        // then we try with xdg-open
        .or_else(|_| try_browser!("xdg-open", url))
        // else do desktop specific stuff
        .or_else(|r| match guess_desktop_env() {
            "kde" => try_browser!("kde-open", url)
                .or_else(|_| try_browser!("kde-open5", url))
                .or_else(|_| try_browser!("kfmclient", "newTab", url)),

            "gnome" => try_browser!("gio", "open", url)
                .or_else(|_| try_browser!("gvfs-open", url))
                .or_else(|_| try_browser!("gnome-open", url)),

            "mate" => try_browser!("gio", "open", url)
                .or_else(|_| try_browser!("gvfs-open", url))
                .or_else(|_| try_browser!("mate-open", url)),

            "xfce" => try_browser!("exo-open", url)
                .or_else(|_| try_browser!("gio", "open", url))
                .or_else(|_| try_browser!("gvfs-open", url)),

            _ => Err(r),
        })
        // at the end, we'll try x-www-browser and return the result as is
        .or_else(|_| try_browser!("x-www-browser", url))
        // if all above failed, map error to not found
        .map_err(|_| {
            Error::new(
                ErrorKind::NotFound,
                "No valid browsers detected. You can specify one in BROWSERS environment variable",
            )
        })
        // and convert a successful result into a ()
        .map(|_| ())
}

#[inline]
fn try_with_browser_env(url: &str) -> Result<()> {
    // $BROWSER can contain ':' delimited options, each representing a potential browser command line
    for browser in std::env::var("BROWSER")
        .unwrap_or_else(|_| String::from(""))
        .split(':')
    {
        if !browser.is_empty() {
            // each browser command can have %s to represent URL, while %c needs to be replaced
            // with ':' and %% with '%'
            let cmdline = browser
                .replace("%s", url)
                .replace("%c", ":")
                .replace("%%", "%");
            let cmdarr: Vec<&str> = cmdline.split_ascii_whitespace().collect();
            let browser_cmd = cmdarr[0];
            let env_exit = for_matching_path(browser_cmd, |pb| {
                let mut cmd = Command::new(pb);
                for arg in cmdarr.iter().skip(1) {
                    cmd.arg(arg);
                }
                if !browser.contains("%s") {
                    // append the url as an argument only if it was not already set via %s
                    cmd.arg(url);
                }
                run_command(&mut cmd, !is_text_browser(pb))
            });
            if env_exit.is_ok() {
                return Ok(());
            }
        }
    }
    Err(Error::new(
        ErrorKind::NotFound,
        "No valid browser configured in BROWSER environment variable",
    ))
}

/// Detect the desktop environment
#[inline]
fn guess_desktop_env() -> &'static str {
    let unknown = "unknown";
    let xcd: String = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_else(|_| unknown.into())
        .to_ascii_lowercase();
    let dsession: String = std::env::var("DESKTOP_SESSION")
        .unwrap_or_else(|_| unknown.into())
        .to_ascii_lowercase();

    if xcd.contains("gnome") || xcd.contains("cinnamon") || dsession.contains("gnome") {
        // GNOME and its derivatives
        "gnome"
    } else if xcd.contains("kde")
        || std::env::var("KDE_FULL_SESSION").is_ok()
        || std::env::var("KDE_SESSION_VERSION").is_ok()
    {
        // KDE: https://userbase.kde.org/KDE_System_Administration/Environment_Variables#Automatically_Set_Variables
        "kde"
    } else if xcd.contains("mate") || dsession.contains("mate") {
        // We'll treat MATE as distinct from GNOME due to mate-open
        "mate"
    } else if xcd.contains("xfce") || dsession.contains("xfce") {
        // XFCE
        "xfce"
    } else {
        // All others
        unknown
    }
}

/// Returns true if specified command refers to a known list of text browsers
#[inline]
fn is_text_browser(pb: &Path) -> bool {
    for browser in TEXT_BROWSERS.iter() {
        if pb.ends_with(&browser) {
            return true;
        }
    }
    false
}

#[inline]
fn for_matching_path<F>(name: &str, op: F) -> Result<()>
where
    F: FnOnce(&PathBuf) -> Result<()>,
{
    let err = Err(Error::new(ErrorKind::NotFound, "command not found"));

    // if the name already includes path separator, we should not try to do a PATH search on it
    // as it's likely an absolutely or relative name, so we treat it as such.
    if name.contains(std::path::MAIN_SEPARATOR) {
        let pb = std::path::PathBuf::from(name);
        if let Ok(metadata) = pb.metadata() {
            if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                return op(&pb);
            }
        } else {
            return err;
        }
    } else {
        // search for this name inside PATH
        if let Ok(path) = std::env::var("PATH") {
            for entry in path.split(':') {
                let mut pb = std::path::PathBuf::from(entry);
                pb.push(name);
                if let Ok(metadata) = pb.metadata() {
                    if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                        return op(&pb);
                    }
                }
            }
        }
    }
    // return the not found err, if we didn't find anything above
    err
}

/// Run the specified command in foreground/background
#[inline]
fn run_command(cmd: &mut Command, background: bool) -> Result<()> {
    if background {
        // if we're in background, set stdin/stdout to null and spawn a child, as we're
        // not supposed to have any interaction.
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
    } else {
        // if we're in foreground, use status() instead of spawn(), as we'd like to wait
        // till completion
        cmd.status().and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "command present but exited unsuccessfully",
                ))
            }
        })
    }
}

static TEXT_BROWSERS: [&str; 9] = [
    "lynx", "links", "links2", "elinks", "w3m", "eww", "netrik", "retawq", "curl",
];
