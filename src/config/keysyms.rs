use crate::errors::WmResult;

use x11::xlib::{Display, XKeycodeToKeysym, XKeysymToKeycode, XKeysymToString, XStringToKeysym};
use x11::keysym::{XK_Super_R, XK_Super_L, XK_Shift_L, XK_Shift_R, XK_Alt_R, XK_Alt_L, XK_Control_R, XK_Control_L, XK_Caps_Lock, XK_Meta_L, XK_Meta_R};
use x11rb::protocol::xproto::Keycode;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Keysym {
    name: String,
    code: Option<u8>,
    value: u64,
}

#[allow(unused)]
impl Keysym {
    /// Given a Keysym name and a Keysym value, create a Keysym, which does no have a keycode.
    ///
    /// You can get the keycode by using the `try_get_keycode` method.
    pub fn new(name: String, value: u64) -> Self {
        Self { name, value, code: None }
    }

    fn new_full(name: String, value: u64, code: Option<u8>) -> Self {
        Self { name, value, code }
    }

    /// Return the string representation of the keysym.
    ///
    /// For example: a, A, backtick, Super_L.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Return the Keycode, from which this Keysym was created.
    ///
    /// If no keycode is avaliable, return 0
    pub fn code(&self) -> u8 {
        if let Some(c) = self.code {
            return c
        }

        0
    }

    /// Return the actuall value of the Keysym, as defined in the `X11/keysymdef.h` header
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Checks if the Keysym is a Modifier key, for example shift, super key or alt.
    #[allow(non_upper_case_globals)]
    pub fn is_mod(&self) -> bool {
        return match self.value() as u32 {
            XK_Super_L | XK_Super_R | XK_Control_L | XK_Control_R | XK_Alt_L | XK_Alt_R | XK_Shift_L | XK_Shift_R => true,
            _ => false,
        };
    }


    /// Returh the mask value of the keysym, if it is a mod key.
    #[allow(non_upper_case_globals)]
    pub fn mod_mask(&self) -> u16 {
        if self.is_mod() {
            return match self.value() as u32 {
                XK_Shift_R | XK_Shift_L => 1 << 0,
                XK_Caps_Lock => 1 << 1,
                XK_Control_R | XK_Control_L => 1 << 2,
                XK_Meta_L | XK_Meta_R => 1 << 3,
                XK_Super_L | XK_Shift_R => 1 << 6,
                _ => 0,
            }
        }

        return 0;
    }

    /// A reverse process of trying to get a Keycode from a Keysym.
    pub fn try_get_keycode(&mut self, dpy: *mut Display) -> WmResult<u8> {
        if self.code() != 0 {
            Ok(self.code())
        } else {
            let code = unsafe { XKeysymToKeycode(dpy, self.value()) };
            self.code = Some(code);
            Ok(self.code())
        }
    }

    /// Given a string, for example 'a', try to create a keysym out of it.
    ///
    /// This function uses the `new` method, which means that the Keysym created this way won't
    /// have a Keycode. Use the `try_get_keysym` method to get it's Keycode.
    pub fn lookup_string<S: AsRef<str>>(dpy: *mut Display, str: S) -> WmResult<Self> {
        let cstring = unsafe { std::ffi::CString::new(str.as_ref()).unwrap() };
        let value =
            unsafe { XStringToKeysym(cstring.as_c_str().as_ptr()) };
        let ptr = unsafe { XKeysymToString(value) };
        if ptr.is_null() {
            return Err("keysym error: XKeysymToString returned a NULL pointer, indicating that the value passed to it was wrong.".into())
        }
        let name = unsafe {std::ffi::CStr::from_ptr(ptr).to_str()?.to_string()};
        Ok(Keysym::new(name, value))
    }
    /// Given a connection to Xlib and a keycode, attempt to get a Keysym.
    pub fn keysym_from_keycode(dpy: *mut Display, keycode: Keycode, mods: i32) -> WmResult<Keysym> {
        let value = unsafe { XKeycodeToKeysym(dpy, keycode, mods) };
        let raw_str = unsafe { XKeysymToString(value) };
        if raw_str.is_null() {
            return Err("keysym error: XKeysymToString returned a NULL pointer, indicating that the value passed to it was wrong.".into())
        }
        let name = unsafe { std::ffi::CStr::from_ptr(raw_str).to_str()?.to_string() };

        Ok(Keysym::new_full(name, value, Some(keycode)))
    }
}

#[cfg(test)]
mod tests {
    use x11::xlib::XOpenDisplay;

    use crate::config::keysyms::Keysym;

    #[test]
    fn lookup() {
        let dpy = unsafe {XOpenDisplay(std::ptr::null())};

        assert!(Keysym::lookup_string(dpy, "Scroll_Lock").is_ok());
        assert!(Keysym::lookup_string(dpy, "control_l").is_err())
    }
}
