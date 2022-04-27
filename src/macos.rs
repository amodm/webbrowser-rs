use crate::{Browser, BrowserOptions, Error, ErrorKind, Result};
use std::process::{Command, Stdio};

mod common;
use common::from_status;

/// Deal with opening of browsers on Mac OS X, using `open` command
#[inline]
pub fn open_browser_internal(
    browser: Browser,
    url_raw: &str,
    options: &BrowserOptions,
) -> Result<()> {
    let url_s: String = match url::Url::parse(url_raw) {
        Ok(u) => u.as_str().into(),
        Err(_) => url_raw.into(),
    };
    let url = &url_s;
    let mut cmd = Command::new("open");
    match browser {
        Browser::Default => {
            // always return true for a dry run for default browser
            if options.dry_run {
                return Ok(());
            }

            run_command(cmd.arg(url), options)
        }
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
                Some(name) => {
                    if options.dry_run {
                        // in case of a dry run, we just check for the existence of the app dir
                        let md = std::fs::metadata(format!("/Applications/{}.app", name));
                        if md.map(|x| x.is_dir()).unwrap_or(false) {
                            Ok(())
                        } else {
                            Err(Error::new(
                                ErrorKind::NotFound,
                                format!("Browser {} not available", name),
                            ))
                        }
                    } else {
                        // run the command only if not dry_run
                        run_command(cmd.arg("-a").arg(name).arg(url), options)
                    }
                }
                None => Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Unsupported browser {:?}", browser),
                )),
            }
        }
    }
}

fn run_command(cmd: &mut Command, options: &BrowserOptions) -> Result<()> {
    if options.suppress_output && option_env!("WEBBROWSER_FORCE_NO_SUPPRESS").is_none() {
        cmd.stdout(Stdio::null())
            .stdin(Stdio::null())
            .stderr(Stdio::null());
    }

    if option_env!("WEBBROWSER_DEBUG_TESTS").is_some() {
        println!("[debug-macos-tests] about to run command: {:?}", cmd);
    }

    from_status(cmd.status())
}
