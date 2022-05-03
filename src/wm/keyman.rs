use crate::config::keysyms::Keysym;
use crate::config::Keybinds;
use crate::errors::WmResult;
use x11::xlib::Display;

use super::actions::Action;

#[derive(Debug, Clone)]
struct ManagedKeybind {
    mask: u16,
    codes: Vec<u8>,
    action: Action,
}

#[derive(Debug, Default)]
pub struct KeyManager {
    managed_keybinds: Vec<ManagedKeybind>,
    keys: Vec<u8>,
    mask: u16,
}

impl KeyManager {
    pub fn init(&mut self, dpy: *mut Display, keybinds: &Keybinds) -> WmResult {
        let mut managed_keybinds: Vec<ManagedKeybind> = Vec::new();

        for (names, action) in keybinds.get_names_and_actions() {
            let mut masked_keys_pair = (0, Vec::new());
            for name in names {
                let mut keysym = Keysym::lookup_string(dpy, name)?;
                if keysym.is_mod() {
                    masked_keys_pair.0 |= keysym.mod_mask();
                    #[cfg(debug_assertions)]
                    println!("mod mask: {}", keysym.name());
                } else {
                    masked_keys_pair.1.push(keysym.try_get_keycode(dpy)?)
                }
            }

            managed_keybinds.push(ManagedKeybind {
                mask: masked_keys_pair.0,
                codes: masked_keys_pair.1,
                action,
            })
        }

        self.managed_keybinds = managed_keybinds;

        Ok(())
    }

    /// Get a list of modifier key masks and a list of key codes.
    /// These values are used to "grab" these keys in the X server.
    pub fn get_grab_codes(
        &self,
        dpy: *mut Display,
        keybinds: &Keybinds,
    ) -> WmResult<Vec<(u16, Vec<u8>)>> {
        let mut ret = Vec::new();
        for each in keybinds.get_names() {
            let mut masked_keys_pair = (0, Vec::new());
            for name in each {
                let mut keysym = Keysym::lookup_string(dpy, name)?;
                if keysym.is_mod() {
                    masked_keys_pair.0 |= keysym.mod_mask();
                } else {
                    masked_keys_pair.1.push(keysym.try_get_keycode(dpy)?)
                }
            }
            ret.push(masked_keys_pair)
        }

        Ok(ret)
    }

    /// What to do on key press.
    pub fn key_press(
        &mut self,
        ev: &x11rb::protocol::xproto::KeyPressEvent,
    ) -> WmResult<Option<Action>> {
        self.keys.push(ev.detail);
        self.mask = ev.state;
        #[cfg(debug_assertions)]
        println!("Keys and mask: {:?}, {}", self.keys, self.mask);

        for keybind in &self.managed_keybinds {
            if self.keys == keybind.codes && self.mask == keybind.mask {
                return Ok(Some(keybind.action.clone()));
            }
        }

        Ok(None)
    }

    pub fn key_release(&mut self, _ev: &x11rb::protocol::xproto::KeyReleaseEvent) -> WmResult {
        self.keys.clear();
        self.mask = 0;
        Ok(())
    }
}
