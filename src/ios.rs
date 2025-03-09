use crate::{Browser, BrowserOptions, Error, ErrorKind, Result, TargetType};
use block2::Block;
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2::{class, msg_send, MainThreadMarker};
use objc2_foundation::{NSDictionary, NSObject, NSString, NSURL};

fn app(mtm: MainThreadMarker) -> Option<Retained<NSObject>> {
    let _ = mtm;
    // SAFETY: The signature is correct, and we hold `MainThreadMarker`, so we
    // know we're on the main thread where it's safe to access the shared
    // UIApplication.
    //
    // NOTE: `sharedApplication` is declared as returning non-NULL, but it
    // will only do so inside `UIApplicationMain`; if called outside, the
    // shared application is NULL.
    unsafe { msg_send![class!(UIApplication), sharedApplication] }
}

fn open_url(
    app: &NSObject,
    url: &NSURL,
    options: &NSDictionary,
    handler: Option<&Block<dyn Fn(Bool)>>,
) {
    unsafe { msg_send![app, openURL: url, options: options, completionHandler: handler] }
}

/// Deal with opening of browsers on iOS/tvOS/visionOS.
///
/// watchOS doesn't have a browser, so this won't work there.
pub(super) fn open_browser_internal(
    _browser: Browser,
    target: &TargetType,
    options: &BrowserOptions,
) -> Result<()> {
    // ensure we're opening only http/https urls, failing otherwise
    let url = target.get_http_url()?;

    let mtm = MainThreadMarker::new().ok_or_else(|| {
        Error::new(
            ErrorKind::Other,
            "Cannot open URL from a thread that is not the main thread",
        )
    })?;

    // always return true for a dry run
    if options.dry_run {
        return Ok(());
    }

    let app = app(mtm).ok_or(Error::new(
        ErrorKind::Other,
        "UIApplication is null, can't open url",
    ))?;

    // Create ns string class from our string
    let url_string = NSString::from_str(url);
    // Create NSURL object with given string
    let url_object = unsafe { NSURL::URLWithString(&url_string) }.ok_or(Error::new(
        ErrorKind::Other,
        "Failed creating NSURL; is the URL valid?",
    ))?;
    // empty options dictionary
    let options = NSDictionary::new();

    // Open url
    open_url(&app, &url_object, &options, None);
    Ok(())
}
