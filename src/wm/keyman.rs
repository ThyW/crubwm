use crate::config::keysyms::Keysym;
use crate::config::{Keybind, Keybinds};
use crate::errors::WmResult;
use x11::xlib::Display;

use super::actions::Action;

#[derive(Debug, Default)]
pub struct KeyManager {
    buff: Vec<Keysym>,
    registered_keybinds: Keybinds,
}

impl KeyManager {
    pub fn set_keybinds(&mut self, binds: Keybinds) {
        self.registered_keybinds = binds;
    }

    pub fn _add_keybinds(&mut self, binds: Vec<Keybind>) {
        self.registered_keybinds._extend(binds)
    }

    pub fn key_press(
        &mut self,
        ev: &x11rb::protocol::xproto::KeyPressEvent,
        dpy: *mut Display,
    ) -> WmResult<Option<Action>> {
        let keysym = Keysym::keysym_from_keycode(dpy, ev.detail, 0)?;
        self.buff.push(keysym);

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
        self.buff.remove(
            self.buff
                .iter()
                .enumerate()
                .find(|k| k.1 == &keysym)
                .unwrap()
                .0,
        );

        println!("removed");

        Ok(())
    }
}
