extern crate widestring;
extern crate winapi;

use crate::{Browser, BrowserOptions, Error, ErrorKind, Result, TargetType};
pub use std::os::windows::process::ExitStatusExt;
use std::{mem, ptr};
use widestring::U16CString;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::combaseapi::{CoInitializeEx, CoUninitialize};
use winapi::um::objbase::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE};
use winapi::um::shellapi::{
    ShellExecuteExW, SEE_MASK_CLASSNAME, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
};
use winapi::um::winuser::SW_SHOWNORMAL;

/// Deal with opening of browsers on Windows, using [`ShellExecuteW`](
/// https://docs.microsoft.com/en-us/windows/desktop/api/shellapi/nf-shellapi-shellexecutew)
/// function.
///
/// We ignore BrowserOptions on Windows, except for honouring [BrowserOptions::dry_run]
pub(super) fn open_browser_internal(
    browser: Browser,
    target: &TargetType,
    options: &BrowserOptions,
) -> Result<()> {
    match browser {
        Browser::Default => {
            // always return true for a dry run for default browser
            if options.dry_run {
                return Ok(());
            }

            static OPEN: &[u16] = &['o' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 0x0000];
            static HTTP: &[u16] = &['h' as u16, 't' as u16, 't' as u16, 'p' as u16, 0x0000];
            let url = U16CString::from_str(target.as_ref())
                .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
            let code = unsafe {
                let coinitializeex_result = CoInitializeEx(
                    ptr::null_mut(),
                    COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
                );
                let mut sei = SHELLEXECUTEINFOW {
                    cbSize: mem::size_of::<SHELLEXECUTEINFOW>() as u32,
                    nShow: SW_SHOWNORMAL,
                    lpFile: url.as_ptr(),
                    fMask: SEE_MASK_CLASSNAME | SEE_MASK_NOCLOSEPROCESS,
                    lpVerb: OPEN.as_ptr(),
                    lpClass: HTTP.as_ptr(),
                    ..Default::default()
                };
                ShellExecuteExW(&mut sei);
                let code = sei.hInstApp as usize as i32;

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
