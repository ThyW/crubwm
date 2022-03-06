use x11rb::{
    connect,
    connection::Connection,
    protocol::xproto::{ConnectionExt, Screen, ConfigureWindowAux, ChangeWindowAttributesAux, EventMask},
    rust_connection::RustConnection,
};

use crate::{errors::WmResult, wm::workspace::Workspaces};

use super::{container::Client, geometry::Geometry, workspace::WorkspaceId};

pub struct State {
    connection: RustConnection,
    screen_index: usize,
    workspaces: Option<Workspaces>,
    focused_workspace: WorkspaceId,
    focused_client: Client,
}

impl State {
    /// Connect to the X server and create WM state.
    ///
    /// If a name of the display is given, use that display, otherwise use the display from the
    /// DISPLAY environmental variable.
    pub fn new(_name: Option<&str>) -> WmResult<Self> {
        let (conn, screen_index) = connect(None)?;

        let change = ChangeWindowAttributesAux::default()
            .event_mask(EventMask::KEY_PRESS | EventMask::SUBSTRUCTURE_NOTIFY);

        conn.change_window_attributes(conn.setup().roots[screen_index].root, &change)?;

        Ok(Self {
            connection: conn,
            screen_index,
            workspaces: None,
            focused_workspace: 0,
            focused_client: 0,
        })
    }

    /// Get the information about the current root of our display.
    pub fn root_screen(&self) -> &Screen {
        &self.connection.setup().roots[self.screen_index]
    }

    /// Return the X window id of the root window
    pub fn root_window(&self) -> u32 {
        self.root_screen().root
    }

    /// Get the geometry of the root window.
    pub fn root_geometry(&self) -> WmResult<Geometry> {
        let cookie = self.connection.get_geometry(self.root_window())?;
        let geometry = cookie.reply()?.into();

        Ok(geometry)
    }

    /// Get a referecnce to the underlying X connection.
    pub fn connection(&self) -> &RustConnection {
        &self.connection
    }
}
