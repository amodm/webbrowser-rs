use super::{BrowserOptions, Error, Result};
use log::debug;
use std::process::{Child, Command, Stdio};

/// Parses a Windows command line `line` into tokens following Windows command-line quoting rules
/// (`CommandLineToArgvW` / UCRT), and invokes `op` on each token.
///
/// This lives in `common` rather than `windows` because it is shared by both the native Windows
/// implementation (`windows.rs`) and the WSL `file://` handler (`unix.rs::wsl`), which runs on
/// Linux (`#[cfg(target_os = "linux")]`) and also parses Windows registry browser commands.
pub(crate) fn for_each_windows_token<F>(line: &str, mut op: F)
where
    F: FnMut(&str),
{
    let mut token = String::new();
    let mut in_quotes = false;
    let mut in_token = false;
    let mut slashes = 0usize;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                slashes += 1;
                in_token = true;
            }
            '"' => {
                for _ in 0..(slashes / 2) {
                    token.push('\\');
                }
                if slashes & 1 == 0 {
                    if in_quotes && chars.peek() == Some(&'"') {
                        chars.next();
                        token.push('"');
                    } else {
                        in_quotes = !in_quotes;
                    }
                } else {
                    token.push('"');
                }
                slashes = 0;
                in_token = true;
            }
            ' ' | '\t' if !in_quotes => {
                for _ in 0..slashes {
                    token.push('\\');
                }
                slashes = 0;
                if in_token {
                    op(&token);
                    token.clear();
                    in_token = false;
                }
            }
            _ => {
                for _ in 0..slashes {
                    token.push('\\');
                }
                slashes = 0;
                token.push(ch);
                in_token = true;
            }
        }
    }

    for _ in 0..slashes {
        token.push('\\');
    }
    if in_token {
        op(&token);
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

#[cfg(test)]
mod token_tests {
    use super::for_each_windows_token;

    fn tokenize(line: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        for_each_windows_token(line, |token| tokens.push(token.to_owned()));
        tokens
    }

    #[test]
    fn test_for_each_token_utf8_boundaries() {
        let cases: &[(&str, &[&str])] = &[
            ("éclair arg", &["éclair", "arg"]),
            ("浏览器 arg", &["浏览器", "arg"]),
            ("🦊fox arg", &["🦊fox", "arg"]),
            ("browser 用户", &["browser", "用户"]),
            ("browser héllo", &["browser", "héllo"]),
            (
                "  浏览器  \"默认 配置\" 🦊 ",
                &["浏览器", "默认 配置", "🦊"],
            ),
            (
                "\"C:\\Program Files\\浏览器.exe\" 用户 \"%1\"",
                &["C:\\Program Files\\浏览器.exe", "用户", "%1"],
            ),
            ("", &[]),
            ("  browser  \"%1\"  ", &["browser", "%1"]),
            ("browser \"\"", &["browser", ""]),
        ];

        for &(line, expected) in cases {
            assert_eq!(tokenize(line), expected, "input: {line:?}");
        }
    }

    #[test]
    fn test_for_each_token_localized_powershell_warning() {
        let line = "WARNING: (0) : 未指定源文件\n\n(1) : using System;";
        assert_eq!(
            tokenize(line),
            [
                "WARNING:",
                "(0)",
                ":",
                "未指定源文件\n\n(1)",
                ":",
                "using",
                "System;",
            ]
        );
    }

    #[test]
    fn test_for_each_token_quoted_flag_values_and_escapes() {
        let cases: &[(&str, &[&str])] = &[
            // Scoop / Chromium quoted flag value with spaces (issue #130)
            (
                r#""C:\Users\User\scoop\apps\helium\current\chrome.exe" --user-data-dir="C:\Users\User\scoop\persist\helium\User Data" --single-argument %1"#,
                &[
                    r"C:\Users\User\scoop\apps\helium\current\chrome.exe",
                    r"--user-data-dir=C:\Users\User\scoop\persist\helium\User Data",
                    "--single-argument",
                    "%1",
                ],
            ),
            (
                r#""C:\chrome.exe" --user-data-dir="C:\Scoop\UserData" "%1""#,
                &[r"C:\chrome.exe", r"--user-data-dir=C:\Scoop\UserData", "%1"],
            ),
            // Quoted directory path ending in escaped trailing backslash (`\\"`)
            (
                r#""C:\chrome.exe" --user-data-dir="C:\Users\User\User Data\\" %1"#,
                &[
                    r"C:\chrome.exe",
                    r"--user-data-dir=C:\Users\User\User Data\",
                    "%1",
                ],
            ),
            // Escaped quotes `\"` inside a quoted flag value must NOT split on inner spaces
            (
                r#"wrapper.exe --args="--profile \"Work Profile\" --incognito" %1"#,
                &[
                    "wrapper.exe",
                    r#"--args=--profile "Work Profile" --incognito"#,
                    "%1",
                ],
            ),
            // Doubled quotes `""` inside a quoted flag value
            (
                r#"wrapper.exe --title="My ""Cool"" Browser" %1"#,
                &["wrapper.exe", r#"--title=My "Cool" Browser"#, "%1"],
            ),
            // Tab-delimited arguments
            (
                "\"C:\\Program Files\\browser.exe\"\t--flag=\"a b\"\t%1",
                &[r"C:\Program Files\browser.exe", "--flag=a b", "%1"],
            ),
            (r#"browser --opt="" %1"#, &["browser", "--opt=", "%1"]),
            (
                r#"browser --opt="hello world"-"foo bar" %1"#,
                &["browser", "--opt=hello world-foo bar", "%1"],
            ),
            (
                r#"cmd.exe /c start "" "%1""#,
                &["cmd.exe", "/c", "start", "", "%1"],
            ),
            (r"browser C:\Temp\ %1", &["browser", r"C:\Temp\", "%1"]),
            (r"browser C:\Temp\", &["browser", r"C:\Temp\"]),
        ];

        for &(line, expected) in cases {
            assert_eq!(tokenize(line), expected, "input: {line:?}");
        }
    }

    /// Verify lossless round-trip against Rust std's Windows `Command` argument encoder.
    #[test]
    fn test_win32_roundtrip_idempotence() {
        fn win32_encode_args(args: &[&str]) -> String {
            let mut cmd = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    cmd.push(' ');
                }
                let quote =
                    arg.is_empty() || arg.bytes().any(|c| c == b' ' || c == b'\t' || c == b'"');
                if quote {
                    cmd.push('"');
                }
                let mut backslashes = 0usize;
                for ch in arg.chars() {
                    if ch == '\\' {
                        backslashes += 1;
                    } else {
                        if ch == '"' {
                            for _ in 0..(backslashes + 1) {
                                cmd.push('\\');
                            }
                        } else {
                            for _ in 0..backslashes {
                                cmd.push('\\');
                            }
                        }
                        backslashes = 0;
                        cmd.push(ch);
                    }
                }
                if quote {
                    for _ in 0..(backslashes * 2) {
                        cmd.push('\\');
                    }
                    cmd.push('"');
                } else {
                    for _ in 0..backslashes {
                        cmd.push('\\');
                    }
                }
            }
            cmd
        }

        let args = &[
            r"C:\Program Files\Wrapper\run.exe",
            r"--dir=C:\Trailing Slash\",
            r#"--nested=--flag "value with spaces" --injected"#,
            r#"about:blank" --gpu-launcher=calc"#,
            "",
            r"\\server\share\path\",
        ];
        let encoded = win32_encode_args(args);
        assert_eq!(tokenize(&encoded), args, "encoded: {encoded}");
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
