use crate::{Browser, BrowserOptions, Error, ErrorKind, Result, TargetType};
use objc2::MainThreadMarker;
use objc2_foundation::{NSDictionary, NSString, NSURL};
use objc2_ui_kit::UIApplication;

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

    // always return true for a dry run
    if options.dry_run {
        return Ok(());
    }

    let mtm = MainThreadMarker::new().ok_or(Error::new(
        ErrorKind::Other,
        "UIApplication must be retrieved on main thread",
    ))?;
    let app = UIApplication::sharedApplication(mtm);

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
    unsafe { app.openURL_options_completionHandler(&url_object, &options, None) };
    Ok(())
}
