use crate::errors::WmResult;

use x11::xlib::{Display, XKeycodeToKeysym, XKeysymToString};
use x11rb::protocol::xproto::Keycode;

pub fn get_keysym(dpy: *mut Display, keycode: Keycode, mods: i32) -> WmResult<u64> {
    let out = unsafe { XKeycodeToKeysym(dpy, keycode, mods) };

    Ok(out)
}

pub fn get_keysym_str(dpy: *mut Display, keysym: u64) -> WmResult<String> {
    let out = unsafe { XKeysymToString(keysym) };

    let mut count = 0;
    let cstr = unsafe { std::ffi::CString::from_raw(out) };

    Ok(String::from_utf8(cstr).unwrap_or("error".to_string()))
}
