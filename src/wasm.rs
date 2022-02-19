use crate::{Browser, BrowserOptions, Error, ErrorKind, Result};

/// Deal with opening a URL in wasm32. This implementation ignores the browser attribute
/// and always opens URLs in the same browser where wasm32 vm is running.
#[inline]
pub fn open_browser_internal(_: Browser, url: &str, options: &BrowserOptions) -> Result<()> {
    // we can override the target by the env var WEBBROWSER_WASM_TARGET at compile time
    let configured_target = option_env!("WEBBROWSER_WASM_TARGET");
    let window = web_sys::window();
    match window {
        Some(w) => {
            let target = configured_target.unwrap_or_else(|| "_blank");
            wasm_console_log(
                &format!("target for url {} detected as {}", url, target),
                options,
            );

            match w.open_with_url_and_target(url, target) {
                Ok(x) => match x {
                    Some(_) => Ok(()),
                    None => {
                        wasm_console_log(POPUP_ERR_MSG, options);
                        Err(Error::new(ErrorKind::Other, POPUP_ERR_MSG))
                    }
                },
                Err(_) => {
                    wasm_console_log("window error while opening url", options);
                    Err(Error::new(ErrorKind::Other, "error opening url"))
                }
            }
        }
        None => Err(Error::new(
            ErrorKind::Other,
            "should have a window in this context",
        )),
    }
}

/// Print to browser console
fn wasm_console_log(_msg: &str, options: &BrowserOptions) {
    #[cfg(all(debug_assertions, feature = "wasm-console"))]
    if !options.suppress_output {
        web_sys::console::log_1(&format!("[webbrowser] {}", &_msg).into());
    }
}

const POPUP_ERR_MSG: &'static str = "popup blocked? window detected, but open_url failed";
