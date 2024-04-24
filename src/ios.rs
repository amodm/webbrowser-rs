use crate::{Browser, BrowserOptions, Error, ErrorKind, Result, TargetType};
use block2::Block;
use objc2::rc::Id;
use objc2::runtime::Bool;
use objc2::{class, msg_send, msg_send_id};
use objc2_foundation::{NSDictionary, NSObject, NSString, NSURL};

fn app() -> Option<Id<NSObject>> {
    unsafe { msg_send_id![class!(UIApplication), sharedApplication] }
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

    // always return true for a dry run
    if options.dry_run {
        return Ok(());
    }

    let app = app().ok_or(Error::new(
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
