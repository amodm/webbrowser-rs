extern crate widestring;
extern crate winapi;

use crate::{Browser, BrowserOptions, Error, ErrorKind, Result};
pub use std::os::windows::process::ExitStatusExt;
use std::ptr;
use widestring::U16CString;

/// Deal with opening of browsers on Windows, using [`ShellExecuteW`](
/// https://docs.microsoft.com/en-us/windows/desktop/api/shellapi/nf-shellapi-shellexecutew)
/// function.
///
/// We ignore BrowserOptions on Windows.
#[inline]
pub fn open_browser_internal(browser: Browser, url: &str, _: &BrowserOptions) -> Result<()> {
    use winapi::shared::winerror::SUCCEEDED;
    use winapi::um::combaseapi::{CoInitializeEx, CoUninitialize};
    use winapi::um::objbase::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE};
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winuser::SW_SHOWNORMAL;
    match browser {
        Browser::Default => {
            static OPEN: &[u16] = &['o' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 0x0000];
            let url =
                U16CString::from_str(url).map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
            let code = unsafe {
                let coinitializeex_result = CoInitializeEx(
                    ptr::null_mut(),
                    COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
                );
                let code = ShellExecuteW(
                    ptr::null_mut(),
                    OPEN.as_ptr(),
                    url.as_ptr(),
                    ptr::null(),
                    ptr::null(),
                    SW_SHOWNORMAL,
                ) as usize as i32;
                if SUCCEEDED(coinitializeex_result) {
                    CoUninitialize();
                }
                code
            };
            if code > 32 {
                Ok(())
            } else {
                Err(Error::last_os_error())
            }
        }
        _ => Err(Error::new(
            ErrorKind::NotFound,
            "Only the default browser is supported on this platform right now",
        )),
    }
}
