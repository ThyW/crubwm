use x11rb::{
    connect,
    connection::Connection,
    protocol::xproto::{
        ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt, EventMask, Screen,
    },
    rust_connection::RustConnection,
};

use crate::{errors::WmResult, wm::workspace::Workspaces};

use super::{
    container::{Client, ContainerId},
    geometry::Geometry,
    workspace::{Workspace, WorkspaceId},
};

pub struct State {
    connection: RustConnection,
    screen_index: usize,
    workspaces: Workspaces,
    focused_workspace: Option<WorkspaceId>,
    // TODO: could this just be a window id?
    focused_client: Option<ContainerId>,
}

impl State {
    /// Connect to the X server and create WM state.
    ///
    /// If a name of the display is given, use that display, otherwise use the display from the
    /// DISPLAY environmental variable.
    pub fn new(_name: Option<&str>) -> WmResult<Self> {
        let (conn, screen_index) = connect(None)?;

        // change root window attributes
        let change = ChangeWindowAttributesAux::default().event_mask(
            EventMask::KEY_PRESS
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::BUTTON_PRESS
                | EventMask::POINTER_MOTION
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::STRUCTURE_NOTIFY
                | EventMask::PROPERTY_CHANGE,
        );

        conn.change_window_attributes(conn.setup().roots[screen_index].root, &change)?;

        Ok(Self {
            connection: conn,
            screen_index,
            workspaces: Vec::new(),
            focused_workspace: None,
            focused_client: None,
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

    /// Go through all workspaces, if they contain a given window: return the reference to the
    /// workspace, otherwise don't return anything.
    fn workspace_for_window(&self, wid: u32) -> Option<&Workspace> {
        for workspace in &self.workspaces {
            if workspace.contains_wid(wid) {
                return Some(&workspace);
            }
        }

        None
    }

    /// Go through all workspaces, if they contain a given window: return a mutable reference to the
    /// workspace, otherwise don't return anything.
    fn workspace_for_window_mut(&mut self, wid: u32) -> Option<&mut Workspace> {
        for workspace in self.workspaces.iter_mut() {
            if workspace.contains_wid(wid) {
                return Some(workspace);
            }
        }

        None
    }

    /// Get a referecnce to the underlying X connection.
    pub fn connection(&self) -> &RustConnection {
        &self.connection
    }

    // TODO: names and ids should be loaded from config.
    pub fn init_workspaces(&mut self) {
        for i in 0..11 {
            self.workspaces.push(Workspace::new(format!("{i}"), i));
        }

        self.focused_workspace = Some(self.workspaces[0].id);
    }

    pub fn get_focused_ws(&mut self) -> WmResult<&Workspace> {
        if let Some(id) = self.focused_workspace {
            if let Some(ws) = self.workspaces.iter().find(|ws| ws.id == id) {
                return Ok(ws);
            }
        }

        Err("workspace could not be found".into())
    }

    pub fn get_focused_ws_mut(&mut self) -> WmResult<&mut Workspace> {
        if let Some(id) = self.focused_workspace {
            if let Some(ws) = self.workspaces.iter_mut().find(|ws| ws.id == id) {
                return Ok(ws);
            }
        }

        Err("workspace could not be found".into())
    }

    // TODO: rework this to work with manage window
    pub fn become_wm(&mut self) -> WmResult {
        // get all the subwindows of the root window
        let root = self.root_window();
        let query_tree_cookie = self.connection().query_tree(root)?;
        let reply = query_tree_cookie.reply()?;

        let mut data: Vec<(u32, Geometry)> = Vec::new();
        let mut geom_cookies = Vec::new();

        for window_id in reply.children {
            geom_cookies.push((window_id, self.connection().get_geometry(window_id)?));
        }

        for (id, cookie) in geom_cookies {
            let geom = cookie.reply()?.into();
            data.push((id, geom))
        }

        self.manage_windows(data)
    }

    /// Let a window be managed by the window manager.
    pub fn manage_window(&mut self, wid: u32, geometry: Geometry) -> WmResult {
        let rg = self.root_geometry()?;
        let ws_container_id = self
            .get_focused_ws_mut()?
            .insert(Client::no_pid(wid, geometry))?;
        self.get_focused_ws_mut()?.apply_layout(rg)?;

        let g = self.get_focused_ws()?.find(ws_container_id)?;
        let g = g.try_borrow()?.data().geometry();

        let aux: ConfigureWindowAux = g.into();

        self.connection().configure_window(wid, &aux)?;
        self.connection().map_window(wid)?;

        Ok(())
    }

    /// Let multiple windows be managed by the window manager.
    pub fn manage_windows(&mut self, data: Vec<(u32, Geometry)>) -> WmResult {
        let rg = self.root_geometry()?;
        let ids = self.get_focused_ws_mut()?.insert_many(
            data.iter()
                .map(|tup| Client::no_pid(tup.0, tup.1))
                .collect(),
        )?;
        self.get_focused_ws_mut()?.apply_layout(rg)?;

        let geometries: Vec<(u32, Geometry)> = self
            .get_focused_ws()?
            .find_many(ids)
            .iter()
            .filter(|x| x.is_some())
            .map(|each| {
                let tup = each.as_ref().unwrap();
                let g = tup.1.try_borrow()?.data().geometry();

                Ok((tup.0, g))
            })
            .filter(|x: &WmResult<_>| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        for each in geometries {
            let aux: ConfigureWindowAux = each.1.into();

            self.connection().configure_window(each.0, &aux)?;
            self.connection().map_window(each.0)?;
        }

        Ok(())
    }

    /// This method is called when a window is destroyed.
    ///
    /// First, start by finding the window than remove it and apply the correct geometries to the
    /// rest of the windows in the workspace.
    pub fn unmanage_window(&mut self, window: u32) -> WmResult {
        let rg = self.root_geometry()?;

        let ws_opt = self.workspace_for_window_mut(window);

        if let Some(ws) = ws_opt {
            ws.remove_wid(window)?;
            ws.apply_layout(rg)?;
            for each in ws.get_all()? {
                let borrowed = each.try_borrow()?;

                let g = borrowed.data().geometry();
                let wid_opt = borrowed.data().wid();

                let aux = g.into();

                if let Some(wid) = wid_opt {
                    self.connection().configure_window(wid, &aux);
                };
            }
        }

        Ok(())
    }
}
