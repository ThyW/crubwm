use x11::xlib::{Display, XOpenDisplay};
use x11rb::{
    connect,
    connection::Connection,
    protocol::xproto::{
        ChangeWindowAttributesAux, ConnectionExt, EventMask, GrabMode, KeyPressEvent,
        KeyReleaseEvent, Screen,
    },
    rust_connection::RustConnection,
};

use crate::{
    config::Keybinds, errors::WmResult, wm::keyman::KeyManager, wm::workspace::Workspaces,
};

use super::{
    actions::Action,
    container::{Client, ClientId, CT_TILING},
    geometry::Geometry,
    workspace::{Workspace, WorkspaceId},
};

pub struct State {
    connection: RustConnection,
    dpy: *mut Display,
    screen_index: usize,
    workspaces: Workspaces,
    focused_workspace: Option<WorkspaceId>,
    focused_client: Option<u32>,
    key_manager: KeyManager,
    last_client_id: ClientId,
}

impl State {
    /// Connect to the X server and create WM state.
    ///
    /// If a name of the display is given, use that display, otherwise use the display from the
    /// DISPLAY environmental variable.
    pub fn new(name: Option<&str>) -> WmResult<Self> {
        let (conn, screen_index) = connect(name)?;
        let display = unsafe { XOpenDisplay(std::ptr::null()) };
        if display.is_null() {
            return Err("x11 error: unable to open a connetion to X server.".into());
        }

        // change root window attributes
        let change = ChangeWindowAttributesAux::default().event_mask(
            /* EventMask::KEY_PRESS
            | EventMask::KEY_RELEASE | */
            EventMask::SUBSTRUCTURE_NOTIFY
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
            dpy: display,
            screen_index,
            workspaces: Vec::new(),
            focused_workspace: None,
            focused_client: None,
            key_manager: KeyManager::default(),
            last_client_id: 0,
        })
    }

    /// Initiate the `KeyManager` with the Keybindings loaded in from a configuration file.
    pub fn init_keyman(&mut self, binds: Keybinds) -> WmResult {
        self.key_manager.set_keybinds(binds);
        let dpy = self.display();
        let codes = self.key_manager.grab_codes(dpy)?;
        self.key_manager.init_mods()?;

        for pair in codes {
            for code in pair.1 {
                #[cfg(debug_assertions)]
                println!("grabbing key: {code} with mod {}", pair.0);
                self.connection().grab_key(
                    true,
                    self.root_window(),
                    pair.0,
                    code,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )?;
            }
        }
        Ok(())
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
                return Some(workspace);
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

    pub fn workspace_with_id(&self, id: u32) -> Option<&Workspace> {
        for ws in &self.workspaces {
            if ws.id == id {
                return Some(&ws);
            }
        }

        return None;
    }

    fn new_client_id(&mut self) -> ClientId {
        self.last_client_id += 1;
        self.last_client_id
    }

    /// Get a referecnce to the underlying X connection.
    pub fn connection(&self) -> &RustConnection {
        &self.connection
    }

    // TODO: names and ids should be loaded from config.
    /// Handle the creation and initiation of workspaces.
    ///
    /// In the future, this method should be loading workspace names, ids and indices from the
    /// Config structure.
    pub fn init_workspaces(&mut self) {
        for i in 0..11 {
            self.workspaces.push(Workspace::new(format!("{i}"), i));
        }

        self.focused_workspace = Some(self.workspaces[0].id);
    }

    // Get a reference to the focused workspace.
    fn get_focused_ws(&self) -> WmResult<&Workspace> {
        if let Some(id) = self.focused_workspace {
            if let Some(ws) = self.workspaces.iter().find(|ws| ws.id == id) {
                return Ok(ws);
            }
        }

        Err("workspace could not be found".into())
    }

    // Get a mutable reference to the focused workspace.
    fn get_focused_ws_mut(&mut self) -> WmResult<&mut Workspace> {
        if let Some(id) = self.focused_workspace {
            if let Some(ws) = self.workspaces.iter_mut().find(|ws| ws.id == id) {
                return Ok(ws);
            }
        }

        Err("workspace could not be found".into())
    }

    /// Become a window manager, take control of all open windows in the X server.
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

    pub fn update_windows(&self, wsid: WorkspaceId) -> WmResult {
        let ws = self.workspace_with_id(wsid);
        if let Some(w) = ws {
            for win in w.get_all()? {
                if let Some(wid) = win.data().wid() {
                    self.connection()
                        .configure_window(wid, &win.data().geometry().into())?;
                }
            }
        }

        Ok(())
    }

    /// Let a window be managed by the window manager.
    pub fn manage_window(&mut self, wid: u32) -> WmResult {
        let geometry = self.connection().get_geometry(wid)?.reply()?.into();
        let rg = self.root_geometry()?;
        let id = self.new_client_id();
        let ws_container_id = self
            .get_focused_ws_mut()?
            .insert(Client::no_pid(wid, geometry, id), CT_TILING);
        self.get_focused_ws_mut()?.apply_layout(rg)?;

        let g = self.get_focused_ws()?.find(ws_container_id)?;
        let g = g.data().geometry();
        #[cfg(debug_assertions)]
        println!("new window geometry: {}", g);

        self.connection()
            .reparent_window(wid, self.root_window(), 0, 0)?;
        let old_event_mask = self
            .connection()
            .get_window_attributes(wid)?
            .reply()?
            .your_event_mask;
        let cwattrs =
            ChangeWindowAttributesAux::new().event_mask(old_event_mask | EventMask::ENTER_WINDOW);
        self.connection().change_window_attributes(wid, &cwattrs)?;
        let wsid = self.get_focused_ws()?.id;
        self.update_windows(wsid)?;
        self.connection().map_window(wid)?;

        Ok(())
    }

    /// Let multiple windows be managed by the window manager.
    ///
    /// For performance sake, this method does not call `manage_window` internally.
    pub fn manage_windows(&mut self, data: Vec<(u32, Geometry)>) -> WmResult {
        let rg = self.root_geometry()?;
        let id = self.new_client_id();
        self.get_focused_ws_mut()?.insert_many(
            data.iter()
                .map(|tup| Client::no_pid(tup.0, tup.1, id))
                .collect(),
            data.iter().map(|_| CT_TILING).collect(),
        );
        self.get_focused_ws_mut()?.apply_layout(rg)?;

        let wsid = self.get_focused_ws()?.id;

        self.update_windows(wsid)?;

        for each in data {
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
        let mut id = 0;

        if let Some(ws) = ws_opt {
            ws.remove_wid(window)?;
            ws.apply_layout(rg)?;
            id = ws.id;
        }

        if let Some(ws) = self.workspace_with_id(id) {
            for each in ws.get_all()? {
                let g = each.data().geometry();
                let wid_opt = each.data().wid();

                let aux = g.into();

                if let Some(wid) = wid_opt {
                    self.connection().configure_window(wid, &aux)?;
                };
            }
        }

        Ok(())
    }

    /// Handle a enter window event.
    ///
    /// This method is responsible for switching input focus to the newly entered window.
    /// In the future, this will also handle the decorators, WM properties and other necessary
    /// things.
    pub fn handle_enter_event(&mut self, window: u32) -> WmResult {
        let _ = self.focused_client.insert(window);
        let mut id = 0;

        if let Some(ws) = self.workspace_for_window(window) {
            id = ws.id
        }

        let _ = self.focused_workspace.insert(id);

        self.connection().set_input_focus(
            x11rb::protocol::xproto::InputFocus::PARENT,
            window,
            x11rb::CURRENT_TIME,
        )?;
        let window_with_input_focus = self.connection().get_input_focus()?.reply()?.focus;
        #[cfg(debug_assertions)]
        println!("focused window is: {window_with_input_focus}");
        self.connection().flush()?;

        Ok(())
    }

    /// Handle a key press event.
    pub fn handle_key_press(&mut self, ev: &KeyPressEvent) -> WmResult {
        let disp = self.display();
        let out = self.key_manager.key_press(ev, disp)?;
        if let Some(action) = out {
            self.do_action(action)?
        }

        Ok(())
    }

    /// Handle a key release event.
    pub fn handle_key_release(&mut self, ev: &KeyReleaseEvent) -> WmResult {
        let d = self.display();
        self.key_manager.key_release(ev, d)?;
        Ok(())
    }

    // Get a pointer to Xlib display structure. This method is used for handling keyboard
    // events(KeyPress and KeyRelease events).
    fn display(&mut self) -> *mut Display {
        self.dpy
    }

    /// Handle the execution of a given action.
    pub fn do_action(&mut self, a: Action) -> WmResult {
        match a {
            Action::Noop => {}
            Action::Kill => self.handle_action_kill()?,
            Action::Goto(_direction) => {
                // self.handle_action_goto(direction)?
            }
            Action::Move(_direction) => {
                // self.handle_action_move(direction)?
            }
            Action::Execute(command) => self.handle_action_execute(command)?,
            Action::Focus(_direction) => {
                // self.handle_action_focus(direction)?
            }
        }

        Ok(())
    }

    fn handle_action_execute(&mut self, command: String) -> WmResult {
        let _ = std::process::Command::new(command).spawn()?;

        Ok(())
    }

    // TODO: should read wm hints for pid and kill the pid
    fn handle_action_kill(&mut self) -> WmResult {
        if let Some(wid) = self.focused_client {
            #[cfg(debug_assertions)]
            println!("killing {wid}");
            self.connection().destroy_subwindows(wid)?;
            self.connection().destroy_window(wid)?;
        }

        Ok(())
    }
}
