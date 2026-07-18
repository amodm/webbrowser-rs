use crate::{Browser, BrowserOptions, Error, ErrorKind, Result, TargetType};
use objc2_app_kit::{NSWorkspace, NSWorkspaceLaunchOptions};
use objc2_foundation::{NSArray, NSDictionary, NSError, NSString, NSURL};

/// Deal with opening of browsers on macOS using the AppKit `NSWorkspace` APIs.
// We intentionally use the deprecated synchronous launch API (see the note at
// the launch site below), so silence the deprecation warnings for this fn.
#[allow(deprecated)]
pub(super) fn open_browser_internal(
    browser: Browser,
    target: &TargetType,
    options: &BrowserOptions,
) -> Result<()> {
    let workspace = NSWorkspace::sharedWorkspace();

    // Resolve the URL of the browser application we want to launch. Rather than
    // relying on hardcoded application paths, we locate applications by their
    // bundle identifiers (or, for the default browser, ask the system which
    // application handles https urls).
    let app_url = match browser {
        Browser::Default => resolve_default_browser(&workspace),
        _ => {
            let bundle_id = bundle_id_for_browser(browser)
                .ok_or_else(|| Error::new(ErrorKind::NotFound, "browser not supported on macos"))?;
            let bundle_id = NSString::from_str(bundle_id);
            workspace.URLForApplicationWithBundleIdentifier(&bundle_id)
        }
    }
    .ok_or_else(|| Error::new(ErrorKind::NotFound, "browser not found"))?;

    // handle dry-run scenario: the application was located above, so we know it
    // exists without actually launching it.
    if options.dry_run {
        log::debug!("dry-run: not actually opening the browser {browser}");
        return Ok(());
    }

    // create the NSURL for the target we want to open
    let target_string = NSString::from_str(target.as_ref());
    let target_url = NSURL::URLWithString(&target_string)
        .ok_or_else(|| Error::other("failed to create target NSURL"))?;
    let urls = NSArray::from_retained_slice(&[target_url]);

    // configure the launch. We use the (deprecated) synchronous launch API
    // rather than the newer completion-handler based one, as the latter relies
    // on the caller running a main run loop to actually dispatch the launch,
    // which isn't the case for library consumers (and breaks in headless
    // environments such as CI). This is the direct equivalent of the Launch
    // Services `LSOpenFromURLSpec` call we previously used.
    let mut launch_options = NSWorkspaceLaunchOptions::Default;
    if options.dont_switch {
        // don't bring the browser to the foreground
        launch_options |= NSWorkspaceLaunchOptions::WithoutActivation;
    }
    let config = NSDictionary::new();

    // launch the browser
    log::trace!("about to start browser: {browser} for {target}");
    let result = unsafe {
        workspace.openURLs_withApplicationAtURL_options_configuration_error(
            &urls,
            &app_url,
            launch_options,
            &config,
        )
    };
    result.map(|_running_app| ()).map_err(map_launch_error)
}

/// Maps an `NSError` returned while launching the browser to our [`Error`] type,
/// preserving the `NotFound`/`PermissionDenied` distinctions where possible.
fn map_launch_error(err: objc2::rc::Retained<NSError>) -> Error {
    // For the `NSOSStatusErrorDomain`, `code` is the underlying Launch Services
    // `OSStatus`. See the `Result Codes` section of:
    // https://developer.apple.com/documentation/coreservices/launch_services
    let kind = match err.code() {
        // -43 is file not found, while -10814 is launch services err code
        -43 | -10814 => ErrorKind::NotFound,
        -10826 => ErrorKind::PermissionDenied,
        _ => ErrorKind::Other,
    };
    Error::new(kind, err.localizedDescription().to_string())
}

/// Determine the URL of the user's default browser, falling back to Safari if
/// detection fails for any reason.
fn resolve_default_browser(workspace: &NSWorkspace) -> Option<objc2::rc::Retained<NSURL>> {
    let https = NSString::from_str("https://");
    let resolved =
        NSURL::URLWithString(&https).and_then(|url| workspace.URLForApplicationToOpenURL(&url));

    match resolved {
        Some(url) => {
            log::trace!("default browser is {url:?}");
            Some(url)
        }
        None => {
            log::error!("failed to get default browser, falling back to Safari");
            let safari = NSString::from_str(SAFARI_BUNDLE_ID);
            workspace.URLForApplicationWithBundleIdentifier(&safari)
        }
    }
}

/// Maps a [`Browser`] to its macOS bundle identifier. Returns `None` for
/// browsers that aren't supported on macOS.
fn bundle_id_for_browser(browser: Browser) -> Option<&'static str> {
    match browser {
        Browser::Firefox => Some("org.mozilla.firefox"),
        Browser::Chrome => Some("com.google.Chrome"),
        Browser::Opera => Some("com.operasoftware.Opera"),
        Browser::Safari => Some(SAFARI_BUNDLE_ID),
        _ => None,
    }
}

/// We assume Safari to be the default browser, if deductions fail for any reason
const SAFARI_BUNDLE_ID: &str = "com.apple.Safari";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_non_existing_browser() {
        let _ = env_logger::try_init();
        if let Err(err) = open_browser_internal(
            Browser::Opera,
            &TargetType::try_from("https://github.com").expect("failed to parse url"),
            &BrowserOptions::default(),
        ) {
            assert_eq!(err.kind(), ErrorKind::NotFound);
        } else {
            panic!("expected opening non-existing browser to fail");
        }
    }

    #[test]
    fn test_existence() {
        let _ = env_logger::try_init();
        assert!(Browser::Safari.exists());
        assert!(!Browser::Opera.exists());
    }
}
