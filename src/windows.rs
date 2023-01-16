use crate::common::{for_each_token, run_command};
use crate::{Browser, BrowserOptions, Error, ErrorKind, Result, TargetType};
use log::{error, trace};
use std::process::Command;
use windows::core::{PCWSTR, PWSTR};
use windows::w;
use windows::Win32::UI::Shell::{AssocQueryStringW, ASSOCF_IS_PROTOCOL, ASSOCSTR_COMMAND};

/// Deal with opening of browsers on Windows.
///
/// We first use [`AssocQueryStringW`](https://learn.microsoft.com/en-us/windows/win32/api/shlwapi/nf-shlwapi-assocquerystringw)
/// function to determine the default browser, and then invoke it with appropriate parameters.
///
/// We ignore BrowserOptions on Windows, except for honouring [BrowserOptions::dry_run]
pub(super) fn open_browser_internal(
    browser: Browser,
    target: &TargetType,
    options: &BrowserOptions,
) -> Result<()> {
    match browser {
        Browser::Default => {
            // always return true for a dry run for default browser
            if options.dry_run {
                return Ok(());
            }

            trace!("trying to figure out default browser command");
            let mut cmdline_u16 = [0_u16; 512];
            let pwstr = PWSTR::from_raw((&mut cmdline_u16) as *mut u16);
            let cmdline = unsafe {
                let mut line_len: u32 = 512;
                AssocQueryStringW(
                    ASSOCF_IS_PROTOCOL,
                    ASSOCSTR_COMMAND,
                    w!("http"),
                    PCWSTR::null(),
                    pwstr,
                    &mut line_len as *mut u32,
                )
                .map_err(err_other)?;

                PCWSTR::from_raw(&cmdline_u16 as *const u16)
                    .to_string()
                    .map_err(err_other)?
            };
            trace!("default browser command: {}", &cmdline);
            let mut cmd = get_browser_cmd(&cmdline, target)?;
            run_command(&mut cmd, true, options)
        }
        _ => Err(Error::new(
            ErrorKind::NotFound,
            "Only the default browser is supported on this platform right now",
        )),
    }
}

fn err_other(err: impl std::fmt::Display) -> Error {
    error!("failed to get default browser: {}", &err);
    Error::new(ErrorKind::Other, "failed to get default browser")
}

/// Given the configured command line `cmdline` in registry, and the given `url`,
/// return the appropriate `Command` to invoke
fn get_browser_cmd(cmdline: &str, url: &TargetType) -> Result<Command> {
    let mut tokens: Vec<String> = Vec::new();
    for_each_token(cmdline, |token: &str| {
        if matches!(token, "%0" | "%1") {
            tokens.push(url.to_string());
        } else {
            tokens.push(token.to_string());
        }
    });
    if tokens.is_empty() {
        Err(Error::new(ErrorKind::NotFound, "invalid command"))
    } else {
        let mut cmd = Command::new(&tokens[0]);
        if tokens.len() > 1 {
            cmd.args(&tokens[1..]);
        }
        Ok(cmd)
    }
}
