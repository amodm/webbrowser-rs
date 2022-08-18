use crate::{Error, ErrorKind, Result};
use objc::{class, msg_send, runtime::Object, sel, sel_impl};

/// Deal with opening of browsers on iOS
#[inline]
pub fn open_browser_internal(url_raw: &str) -> Result<()> {
    let url_s: String = match url::Url::parse(url_raw) {
        Ok(u) => u.as_str().into(),
        Err(_) => url_raw.into(),
    };
    unsafe {
        let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
        if app.is_null() {
            return Err(Error::new(
                ErrorKind::Other,
                "UIApplication is null, can't open url",
            ));
        }
        let url_cstr = std::ffi::CString::new(url_s).unwrap();
        // Create ns string class from our string
        let url_string: *mut Object = msg_send![class!(NSString), stringWithUTF8String: url_cstr];
        // Create NSURL object with given string
        let url_object: *mut Object = msg_send![class!(NSURL), URLWithString: url_string];
        // No completion handler
        let null_ptr = 0 as *mut Object;
        // Open url
        let () = msg_send![app, openURL: url_object options: {} completionHandler: null_ptr];
        Ok(())
    }
}
