use super::{BrowserOptions, Error, Result};
use log::debug;
use std::process::{Child, Command, Stdio};

/// Parses `line` to find tokens (including quoted strings), and invokes `op`
/// on each token
pub(crate) fn for_each_token<F>(line: &str, mut op: F)
where
    F: FnMut(&str),
{
    let mut start: Option<usize> = None;
    let mut in_quotes = false;
    let mut idx = 0;
    for ch in line.chars() {
        idx += ch.len_utf8();
        match ch {
            '"' => {
                if let Some(start_idx) = start {
                    op(&line[start_idx..idx - 1]);
                    start = None;
                    in_quotes = false;
                } else {
                    start = Some(idx);
                    in_quotes = true;
                }
            }
            ' ' => {
                if !in_quotes {
                    if let Some(start_idx) = start {
                        op(&line[start_idx..idx - 1]);
                        start = None;
                    }
                }
            }
            _ => {
                if start.is_none() {
                    start = Some(idx - 1);
                }
            }
        }
    }
    if let Some(start_idx) = start {
        op(&line[start_idx..idx]);
    }
}

/// Wait for a background child on a detached thread, so that we don't block the caller,
/// but the child still gets reaped once it exits. Without this, on unix, an exited child
/// stays around as a zombie (defunct) process until the parent process exits.
#[cfg(unix)]
fn reap_in_background(mut child: Child) {
    let pid = child.id();
    let spawned = std::thread::Builder::new()
        .name(format!("webbrowser-reap-{pid}"))
        .spawn(move || match child.wait() {
            Ok(status) => debug!("background process {pid} exited with {status}"),
            Err(err) => debug!("failed to wait for background process {pid}: {err}"),
        });
    if let Err(err) = spawned {
        // we can't do much here, as the browser has already been launched successfully
        debug!("failed to spawn reaper thread for background process {pid}: {err}");
    }
}

#[cfg(not(unix))]
fn reap_in_background(_child: Child) {}

/// Run the specified command in foreground/background
pub(crate) fn run_command(
    cmd: &mut Command,
    background: bool,
    options: &BrowserOptions,
) -> Result<()> {
    // if dry_run, we return a true, as executable existence check has
    // already been done
    if options.dry_run {
        debug!("dry-run enabled, so not running: {cmd:?}");
        return Ok(());
    }

    if background {
        debug!("background spawn: {cmd:?}");
        // if we're in background, set stdin/stdout to null and spawn a child, as we're
        // not supposed to have any interaction.
        if options.suppress_output {
            cmd.stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
        } else {
            cmd
        }
        .spawn()
        .map(reap_in_background)
    } else {
        debug!("foreground exec: {cmd:?}");
        // if we're in foreground, use status() instead of spawn(), as we'd like to wait
        // till completion.
        // We also specifically don't suppress anything here, because we're running here
        // most likely because of a text browser
        cmd.status().and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(Error::other("command present but exited unsuccessfully"))
            }
        })
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// Background children must get reaped once they exit, instead of being left
    /// behind as zombie processes. See issue #126
    #[test]
    fn test_background_child_is_reaped() {
        let child = Command::new("true")
            .spawn()
            .expect("failed to spawn `true`");
        let proc_path = format!("/proc/{}", child.id());
        reap_in_background(child);

        // an unreaped child stays in /proc (in zombie state) even after it has exited
        let deadline = Instant::now() + Duration::from_secs(10);
        while std::path::Path::new(&proc_path).exists() {
            assert!(
                Instant::now() < deadline,
                "background child was not reaped: {proc_path} still exists"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
