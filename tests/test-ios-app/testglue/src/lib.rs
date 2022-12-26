use std::ffi::CStr;
use std::os::raw::c_char;

/// Glue function to interface the test app with webbrowser
///
/// # Safety
/// Just satisfying clippy here
#[no_mangle]
pub unsafe extern "C" fn test_open_webbrowser(url_c_chars: *const c_char) {
    let url_c_str = unsafe { CStr::from_ptr(url_c_chars) };
    let url = url_c_str.to_str().expect("not valid utf-8");
    let _ = webbrowser::open(url);
}
