use crate::errors::WmResult;

use x11::xlib::{Display, XKeycodeToKeysym, XKeysymToString};
use x11rb::protocol::xproto::Keycode;

pub fn get_keysym(dpy: *mut Display, keycode: Keycode, mods: i32) -> WmResult<u64> {
    let out = unsafe { XKeycodeToKeysym(dpy, keycode, mods) };

    Ok(out)
}

pub fn get_keysym_str(keysym: u64) -> WmResult<String> {
    let out = unsafe { XKeysymToString(keysym) };

    let cstr = unsafe { std::ffi::CStr::from_ptr(out) };

    Ok(cstr.to_str()?.to_string())
}
