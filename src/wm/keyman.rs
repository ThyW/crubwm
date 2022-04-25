use crate::config::keysyms::Keysym;
use crate::config::{Keybind, Keybinds};
use crate::errors::WmResult;
use x11::xlib::Display;

use super::actions::Action;

#[derive(Debug, Default)]
pub struct KeyManager {
    buff: Vec<Keysym>,
    mod_keysyms: Vec<Keysym>,
    registered_keybinds: Keybinds,
}

impl KeyManager {
    pub fn grab_codes(&self, dpy: *mut Display) -> WmResult<Vec<(u16, Vec<u8>)>> {
        let mut ret = Vec::new();
        for each in self.registered_keybinds.get_names() {
            let mut x = (0, Vec::new());
            for name in each {
                let mut keysym = Keysym::lookup_string(dpy, name)?;
                if keysym.is_mod() {
                    x.0 |= keysym.mod_mask();
                } else {
                    x.1.push(keysym.try_get_keycode(dpy)?)
                }
            }
            ret.push(x)
        }

        Ok(ret)
    }
    pub fn set_keybinds(&mut self, binds: Keybinds) {
        self.registered_keybinds = binds;
    }

    pub fn _add_keybinds(&mut self, binds: Vec<Keybind>) {
        self.registered_keybinds._extend(binds)
    }

    pub fn init_mods(&mut self) -> WmResult {
        self.mod_keysyms = Keysym::init_mods()?;

        Ok(())
    }

    pub fn key_press(
        &mut self,
        ev: &x11rb::protocol::xproto::KeyPressEvent,
        dpy: *mut Display,
    ) -> WmResult<Option<Action>> {
        let keysym = Keysym::keysym_from_keycode(dpy, ev.detail, 0)?;

        for mod_key in &self.mod_keysyms {
            if (mod_key.mod_mask() & ev.state) != 0 {
                #[cfg(debug_assertions)]
                println!("pushing mod: {}", mod_key.name());
                self.buff.push(mod_key.clone());
                break;
            }
        }

        if !self.buff.contains(&keysym) {
            #[cfg(debug_assertions)]
            println!("pushing keysym: {}", keysym.name());
            self.buff.push(keysym);
        }

        // check, if any of the registered keybinds have been satisfied
        let buff_names: Vec<String> = self.buff.iter().map(|k| k.name()).collect();
        let kb_names = self.registered_keybinds.get_names_and_actions();

        for each in kb_names {
            if each.0 == buff_names {
                return Ok(Some(each.1));
            }
        }

        Ok(None)
    }

    pub fn key_release(
        &mut self,
        ev: &x11rb::protocol::xproto::KeyReleaseEvent,
        dpy: *mut Display,
    ) -> WmResult {
        let keysym = Keysym::keysym_from_keycode(dpy, ev.detail, 0)?;
        let mut mods = Vec::new();

        for mod_key in &self.mod_keysyms {
            if (mod_key.mod_mask() & ev.state) != 0 {
                mods.push(mod_key.clone());
                break;
            }
        }

        // TODO: this can probably be done better, but it shouldn't be too much of a performance
        // problem to do this in O(n^2)
        let mut to_remove = Vec::new();
        for (ii, each) in self.buff.iter_mut().enumerate() {
            if each.value() == keysym.value() {
                to_remove.push(ii)
            }

            for mod_key in &mods {
                if mod_key.value() == each.value() {
                    to_remove.push(ii)
                }
            }
        }

        self.buff.clear();

        Ok(())
    }
}
