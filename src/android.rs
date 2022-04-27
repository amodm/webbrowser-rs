use crate::{Browser, BrowserOptions, Error, ErrorKind, Result};
use jni::objects::JValue;
pub use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

/// Deal with opening of browsers on Android. Only [Browser::Default] is supported, and
/// in options, only [BrowserOptions::dry_run] is honoured.
#[inline]
pub fn open_browser_internal(browser: Browser, url: &str, options: &BrowserOptions) -> Result<()> {
    match browser {
        Browser::Default => open_browser_default(url, options),
        _ => Err(Error::new(
            ErrorKind::NotFound,
            "only default browser supported",
        )),
    }
}

/// Open the default browser
#[inline]
pub fn open_browser_default(url: &str, options: &BrowserOptions) -> Result<()> {
    // always return true for a dry run
    if options.dry_run {
        return Ok(());
    }

    // Create a VM for executing Java calls
    let native_activity = ndk_glue::native_activity();
    let vm_ptr = native_activity.vm();
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }.unwrap();
    let env = vm.attach_current_thread().map_err(|_| -> Error {
        Error::new(ErrorKind::Other, "Failed to attach current thread")
    })?;

    // Create ACTION_VIEW object
    let intent_class = env
        .find_class("android/content/Intent")
        .map_err(|_| -> Error { Error::new(ErrorKind::NotFound, "Failed to find Intent class") })?;
    let action_view = env
        .get_static_field(intent_class, "ACTION_VIEW", "Ljava/lang/String;")
        .map_err(|_| -> Error {
            Error::new(ErrorKind::NotFound, "Failed to get intent.ACTION_VIEW")
        })?;

    // Create Uri object
    let uri_class = env
        .find_class("android/net/Uri")
        .map_err(|_| -> Error { Error::new(ErrorKind::NotFound, "Failed to find Uri class") })?;
    let url = env
        .new_string(url)
        .map_err(|_| -> Error { Error::new(ErrorKind::Other, "Failed to create JNI string") })?;
    let uri = env
        .call_static_method(
            uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(*url)],
        )
        .map_err(|_| -> Error { Error::new(ErrorKind::Other, "Failed to parse JNI Uri") })?;

    // Create new ACTION_VIEW intent with the uri
    let intent = env
        .alloc_object(intent_class)
        .map_err(|_| -> Error { Error::new(ErrorKind::Other, "Failed to allocate intent") })?;
    env.call_method(
        intent,
        "<init>",
        "(Ljava/lang/String;Landroid/net/Uri;)V",
        &[action_view, uri],
    )
    .map_err(|_| -> Error { Error::new(ErrorKind::Other, "Failed to initialize intent") })?;

    // Start the intent activity.
    env.call_method(
        native_activity.activity(),
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(intent)],
    )
    .map_err(|_| -> Error { Error::new(ErrorKind::Other, "Failed to start activity") })?;

    Ok(())
}
