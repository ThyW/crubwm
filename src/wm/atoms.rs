use crate::errors::WmResult;

use std::collections::HashMap;

use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

pub struct AtomManager;

impl AtomManager {
    /// Initialize all atoms.
    pub fn init_atoms(c: &RustConnection) -> WmResult<HashMap<String, u32>> {
        let mut hm = HashMap::new();
        // https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints
        let atoms = [
            // root window
            "_NET_SUPPORTED",
            "_NET_CLIENT_LIST",
            "_NET_NUMBER_OF_DESKTOPS",
            "_NET_DESKTOP_GEOMETRY",
            "_NET_DESKTOP_VIEWPORT",
            "_NET_CURRENT_DESKTOP",
            "_NET_DESKTOP_NAMES",
            "_NET_ACTIVE_WINDOW",
            "_NET_WORKAREA",
            "_NET_SUPPORTING_WM_CHECK",
            "_NET_VIRTUAL_ROOTS",
            "_NET_DESKTOP_LAYOUT",
            "_NET_SHOWING_DESKTOP",
            // client messages
            "_NET_WM_STATE",
            "_NET_ACTIVE_WINDOW",
            "_NET_SHOWING_DESKTOP",
            "_NET_CLOSE_WINDOW",
            "_NET_WM_MOVERESIZE",
            "_NET_MOVERESIZE_WINDOW",
            "_NET_REQUEST_FRAME_EXTENTS",
            "_NET_WM_FULLSCREEN_MONITORS",
            "_NET_RESTACK_WINDOW",
            "_NET_CURRENT_DESKTOP",
            "_NET_NUMBER_OF_DESKTOPS",
            "_NET_DESKTOP_GEOMETRY",
            "_NET_DESKTOP_VIEWPORT",
            // window properties
            "_NET_WM_NAME",
            "_NET_WM_VISIBLE_NAME",
            "_NET_WM_ICON_NAME",
            "_NET_WM_VISIBLE_ICON_NAME",
            "_NET_WM_DESKTOP",
            "_NET_WM_WINDOW_TYPE",
            "_NET_WM_STATE",
            "_NET_WM_ALLOWED_ACTIONS",
            "_NET_WM_STRUT",
            "_NET_WM_STRUT_PARTIAL",
            "_NET_WM_ICON_GEOMETRY",
            "_NET_WM_ICON",
            "_NET_WM_PID",
            "_NET_WM_HANDLED_ICONS",
            "_NET_WM_USER_TIME",
            "_NET_WM_USER_TIME_WINDOW",
            "_NET_FRAME_EXTENTS",
            "_NET_WM_OPAQUE_REGION",
            "_NET_WM_BYPASS_COMPOSITOR",
        ];

        for atom in atoms {
            let atom_value = c
                .intern_atom(false, "_NET_WM_PID".as_bytes())?
                .reply()?
                .atom;
            if atom_value == 0 {
                return Err(format!(
                    "x11 atom error: intern atom failed return ATOM_NONE for atom {atom}."
                )
                .into());
            }

            hm.insert(atom.into(), atom_value);
        }

        Ok(hm)
    }
}
