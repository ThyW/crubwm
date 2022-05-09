use x11::xlib::{Display, XOpenDisplay};
use x11rb::{
    connect,
    connection::Connection,
    protocol::xproto::{
        AtomEnum, ButtonIndex, ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt,
        EventMask, GrabMode, InputFocus, KeyPressEvent, KeyReleaseEvent, Screen, StackMode,
    },
    rust_connection::RustConnection,
    CURRENT_TIME, NONE,
};

use crate::{
    config::Keybinds,
    errors::{Error, WmResult},
    wm::actions::{Action, Direction},
    wm::atoms::AtomManager,
    wm::container::{Client, ClientId, CT_MASK_TILING},
    wm::focus_stack::FocusStack,
    wm::geometry::Geometry,
    wm::keyman::KeyManager,
    wm::layouts::LayoutMask,
    wm::workspace::Workspaces,
    wm::workspace::{Workspace, WorkspaceId},
};

use std::{collections::HashMap, rc::Rc};

pub struct State {
    connection: Rc<RustConnection>,
    dpy: *mut Display,
    screen_index: usize,
    workspaces: Workspaces,
    focused_workspace: Option<WorkspaceId>,
    client_focus: FocusStack,
    key_manager: KeyManager,
    last_client_id: ClientId,
    atoms: HashMap<String, u32>,
    is_dragging: bool,
    is_resizing: bool,
}

const ANY_KEY_MASK: u8 = 0;
const ANY_MOD_KEY_MASK: u16 = 32768;
const MIN_WIDTH: u16 = 160;
const MIN_HEIGHT: u16 = 90;

impl State {
    /// Connect to the X server and create WM state.
    ///
    /// If a name of the display is given, use that display, otherwise use the display from the
    /// DISPLAY environmental variable.
    pub fn new(name: Option<&str>) -> WmResult<Self> {
        let (connection, screen_index) = connect(name)?;
        let display = unsafe {
            if let Some(name_string) = name {
                let c_string = std::ffi::CString::new(name_string)?;
                XOpenDisplay(c_string.as_ptr())
            } else {
                XOpenDisplay(std::ptr::null())
            }
        };
        if display.is_null() {
            return Err("x11 error: unable to open a connetion to X server.".into());
        }

        // change root window attributes
        let change = ChangeWindowAttributesAux::default().event_mask(
            EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::STRUCTURE_NOTIFY
                | EventMask::PROPERTY_CHANGE
                | EventMask::FOCUS_CHANGE,
        );

        let root_window = connection.setup().roots[screen_index as usize].root;
        connection.change_window_attributes(root_window, &change)?;
        connection.flush()?;

        let atoms = AtomManager::init_atoms(&connection)?;

        Ok(Self {
            connection: Rc::new(connection),
            dpy: display,
            screen_index,
            workspaces: Vec::new(),
            focused_workspace: None,
            client_focus: FocusStack::new(root_window),
            key_manager: KeyManager::default(),
            last_client_id: 0,
            atoms,
            is_dragging: false,
            is_resizing: false,
        })
    }

    /// Initiate the `KeyManager` with the Keybindings loaded in from a configuration file.
    pub fn init_keyman(&mut self, binds: Keybinds) -> WmResult {
        let dpy = self.display();
        self.key_manager.init(dpy, &binds)?;

        // ungrab any key with any modifier
        self.connection()
            .ungrab_key(ANY_KEY_MASK, self.root_window(), ANY_MOD_KEY_MASK)?;

        for (mask, keycodes) in self.key_manager.get_codes_to_grab(dpy, &binds)? {
            for code in keycodes {
                self.connection().grab_key(
                    true,
                    self.root_window(),
                    mask,
                    code,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )?;
            }
        }
        Ok(())
    }

    /// Get the information about the current root of our display.
    fn root_screen(&self) -> &Screen {
        &self.connection.setup().roots[self.screen_index]
    }

    /// Return the X window id of the root window
    fn root_window(&self) -> u32 {
        self.root_screen().root
    }

    /// Get the geometry of the root window.
    fn root_geometry(&self) -> WmResult<Geometry> {
        let geometry_cookie = self.connection.get_geometry(self.root_window())?;
        let geometry = geometry_cookie.reply()?.into();

        Ok(geometry)
    }

    /// Go through all workspaces, if they contain a given window: return the reference to the
    /// workspace, otherwise don't return anything.
    fn workspace_for_window(&self, wid: u32) -> Option<&Workspace> {
        for workspace in &self.workspaces {
            if workspace.contains_window(wid) {
                return Some(workspace);
            }
        }

        None
    }

    // Get a pointer to Xlib display structure. This method is used for handling keyboard
    // events(KeyPress and KeyRelease events).
    fn display(&mut self) -> *mut Display {
        self.dpy
    }

    /// Go through all workspaces, if they contain a given window: return a mutable reference to the
    /// workspace, otherwise don't return anything.
    fn workspace_for_window_mut(&mut self, wid: u32) -> Option<&mut Workspace> {
        for workspace in self.workspaces.iter_mut() {
            if workspace.contains_window(wid) {
                return Some(workspace);
            }
        }

        None
    }

    /// Search for and return a reference to a workspace with the given workspace id.
    fn workspace_with_id<I: Into<WorkspaceId> + Copy>(&self, id: I) -> Option<&Workspace> {
        for workspace in &self.workspaces {
            if workspace.id == id.into() {
                return Some(workspace);
            }
        }

        None
    }

    /// Search for and return a reference to a workspace with the given workspace id.
    fn workspace_with_id_mut<I: Into<WorkspaceId>>(&mut self, id: I) -> Option<&mut Workspace> {
        let id = id.into();
        for workspace in &mut self.workspaces {
            if workspace.id == id.into() {
                return Some(workspace);
            }
        }

        None
    }

    /// Generate a new client identifier.
    fn new_client_id(&mut self) -> ClientId {
        self.last_client_id += 1;
        self.last_client_id
    }

    /// Get a referecnce to the underlying X connection.
    pub fn connection(&self) -> Rc<RustConnection> {
        self.connection.clone()
    }

    // TODO: names and ids should be loaded from config.
    /// Handle the creation and initiation of workspaces.
    ///
    /// In the future, this method should be loading workspace names, ids and indices from the
    /// Config structure.
    pub fn init_workspaces(&mut self) {
        for i in 1..11 {
            self.workspaces
                .push(Workspace::new(format!("{i}"), i, LayoutMask::ALL));
        }

        self.focused_workspace = Some(self.workspaces[0].id);
    }

    /// Get a reference to the focused workspace.
    fn get_focused_workspace(&self) -> WmResult<&Workspace> {
        if let Some(id) = self.focused_workspace {
            if let Some(ws) = self.workspaces.iter().find(|ws| ws.id == id) {
                return Ok(ws);
            }
        }

        Err("focused workspace could not be found".into())
    }

    /// Get a mutable reference to the focused workspace.
    fn get_focused_workspace_mut(&mut self) -> WmResult<&mut Workspace> {
        if let Some(id) = self.focused_workspace {
            if let Some(ws) = self.workspaces.iter_mut().find(|ws| ws.id == id) {
                return Ok(ws);
            }
        }

        Err("focused workspace could not be found".into())
    }

    /// Become a window manager, take control of all open windows on the X server.
    pub fn become_wm(&mut self) -> WmResult {
        let connection = self.connection();
        let root_window = self.root_window();
        let query_tree_cookie = connection.query_tree(root_window)?;
        let query_tree_reply = query_tree_cookie.reply()?;

        let mut windows_with_geometries: Vec<(u32, Geometry)> = Vec::new();
        let mut geom_cookies = Vec::new();

        for window_id in query_tree_reply.children {
            geom_cookies.push((window_id, connection.get_geometry(window_id)?));
        }

        for (id, cookie) in geom_cookies {
            let geom = cookie.reply()?.into();
            windows_with_geometries.push((id, geom))
        }

        self.manage_windows(windows_with_geometries)
    }

    /// Let a window be managed by the window manager.
    pub fn manage_window(&mut self, window: u32) -> WmResult {
        let connection = self.connection();
        let geometry = self.connection().get_geometry(window)?.reply()?;
        let root_window_geometry = self.root_geometry()?;
        let new_client_id = self.new_client_id();

        self.get_focused_workspace_mut()?.insert_client(
            Client::new_without_process_id(window, geometry, new_client_id),
            CT_MASK_TILING,
        );
        let old_event_mask = self
            .connection()
            .get_window_attributes(window)?
            .reply()?
            .your_event_mask;
        let cw_attributes = ChangeWindowAttributesAux::new()
            .event_mask(old_event_mask | EventMask::ENTER_WINDOW | EventMask::FOCUS_CHANGE);
        self.connection()
            .change_window_attributes(window, &cw_attributes)?;

        let mask: u32 =
            (EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::BUTTON_MOTION).into();

        self.connection().grab_button(
            true,
            window,
            mask as u16,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
            self.root_window(),
            NONE,
            ButtonIndex::M1,
            8u16,
        )?;

        self.connection().grab_button(
            true,
            window,
            mask as u16,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
            self.root_window(),
            NONE,
            ButtonIndex::M3,
            8u16,
        )?;

        self.connection()
            .reparent_window(window, self.root_window(), 0, 0)?;
        self.connection().map_window(window)?;
        self.get_focused_workspace_mut()?
            .apply_layout(root_window_geometry, connection)?;

        self.connection()
            .set_input_focus(InputFocus::NONE, window, CURRENT_TIME)?;
        self.client_focus.set_focused_client(window);

        Ok(())
    }

    /// Let multiple windows be managed by the window manager.
    ///
    /// For performance sake, this method does not call `manage_window` internally.
    pub fn manage_windows(&mut self, windows_and_geometries: Vec<(u32, Geometry)>) -> WmResult {
        let connection = self.connection();
        let root_window_geometry = self.root_geometry()?;
        let new_client_ids = (0usize..windows_and_geometries.len())
            .map(|_| self.new_client_id())
            .collect::<Vec<u64>>();
        self.get_focused_workspace_mut()?.insert_many(
            windows_and_geometries
                .iter()
                .enumerate()
                .map(|(index, (window, geometry))| {
                    Client::new_without_process_id(*window, *geometry, new_client_ids[index])
                })
                .collect(),
            windows_and_geometries
                .iter()
                .map(|_| CT_MASK_TILING)
                .collect(),
        );
        self.get_focused_workspace_mut()?
            .apply_layout(root_window_geometry, connection)?;

        for (window, _) in windows_and_geometries {
            self.connection().map_window(window)?;
        }

        Ok(())
    }

    /// This method is called when a window is destroyed.
    ///
    /// First, start by finding the window than remove it and apply the correct geometries to the
    /// rest of the windows in the workspace.
    pub fn unmanage_window(&mut self, window: u32) -> WmResult {
        let root_geometry = self.root_geometry()?;
        let connection = self.connection();

        let workspace_option = self.workspace_for_window_mut(window);

        if let Some(workspace) = workspace_option {
            workspace.remove_window(window)?;
            workspace.apply_layout(root_geometry, connection)?;
        }

        // set input focus to previously focused client
        if let Some(previous_window_id) = self.client_focus.previously_focused_client() {
            self.connection().set_input_focus(
                InputFocus::NONE,
                previous_window_id,
                CURRENT_TIME,
            )?;
            self.client_focus.remove_client(window);
        }

        Ok(())
    }

    /// Handle an enter window event.
    ///
    /// This method is responsible for switching input focus to the newly entered window.
    /// In the future, this will also handle the decorators, WM properties and other necessary
    /// things.
    pub fn handle_enter_event(&mut self, window: u32) -> WmResult {
        self.client_focus.set_focused_client(window);
        let mut id = self.get_focused_workspace()?.id;

        if let Some(workspace) = self.workspace_for_window(window) {
            id = workspace.id
        }

        let _ = self.focused_workspace.insert(id);

        /* let configure_window = ConfigureWindowAux::new().stack_mode(Some(StackMode::TOP_IF));
        self.connection().configure_window(window, &configure_window)?; */
        self.connection().set_input_focus(
            x11rb::protocol::xproto::InputFocus::NONE,
            window,
            x11rb::CURRENT_TIME,
        )?;

        self.connection().flush()?;

        Ok(())
    }

    /// Handle a key press event.
    pub fn handle_key_press(&mut self, ev: &KeyPressEvent) -> WmResult {
        let action_option = self.key_manager.on_key_press(ev)?;
        if let Some(action) = action_option {
            self.do_action(action)?
        }

        Ok(())
    }

    /// Handle a key release event.
    pub fn handle_key_release(&mut self, ev: &KeyReleaseEvent) -> WmResult {
        self.key_manager.on_key_release(ev)?;
        Ok(())
    }

    /// Handle a button press event.
    ///
    /// We check which button on the mouse was pressed, if it was the left button(ev.detail = 1), we know that the
    /// user wants to move this client around, we set the `is_dragging` filed to true. If, on the
    /// other hand, the right button(ev.detail = 3) was pressed, we know the user wants to resize
    /// the window and we set the `is_resizing` flag to to true.
    pub fn handle_button_press(
        &mut self,
        ev: &x11rb::protocol::xproto::ButtonPressEvent,
    ) -> WmResult {
        let workspace = self.workspace_for_window_mut(ev.event).ok_or_else(|| {
            Error::Generic(format!(
                "workspace error: unable to find workspace for window id {}",
                ev.event
            ))
        })?;

        let container = workspace.find_by_window_id_mut(ev.event)?;

        if !container.is_floating() {
            return Ok(());
        } else {
            container.change_last_position((ev.root_x, ev.root_y));
            match ev.detail {
                1 => self.is_dragging = true,
                3 => self.is_resizing = true,
                _ => (),
            };
        }

        Ok(())
    }

    /// Handle a button release event.
    ///
    /// Very similar to button press, we check the `ev.detail` field of the `ButtonReleaseEvent`,
    /// and either finish the dragging process, updating the window's position for the final time,
    /// or we finish the resizing process, updating the window's width and height for the final
    /// time.
    pub fn handle_button_release(
        &mut self,
        ev: &x11rb::protocol::xproto::ButtonReleaseEvent,
    ) -> WmResult {
        let connection = self.connection();
        let workspace = self.workspace_for_window_mut(ev.event).ok_or_else(|| {
            Error::Generic(format!(
                "workspace error: unable to find workspace for window id {}",
                ev.event
            ))
        })?;

        let container = workspace.find_by_window_id_mut(ev.event)?;

        if !container.is_floating() {
            return Ok(());
        } else {
            match ev.detail {
                1 => {
                    let last_event_position = container.last_position().unwrap();
                    let diff = (
                        last_event_position.0 as i16 - ev.root_x,
                        last_event_position.1 as i16 - ev.root_y,
                    );
                    match container.data_mut() {
                        crate::wm::container::ContainerType::Floating(c) => {
                            c.geometry.x -= diff.0;
                            c.geometry.y -= diff.1;

                            connection.configure_window(c.window_id(), &c.geometry().into())?;
                        }
                        _ => (),
                    }
                    self.is_dragging = false
                }
                3 => {
                    let last_event_position = container.last_position().unwrap();
                    let diff = (
                        last_event_position.0 as i16 - ev.root_x,
                        last_event_position.1 as i16 - ev.root_y,
                    );
                    let geom = container.data().geometry();
                    let (w, h) = (geom.width as i16 - diff.0, geom.height as i16 - diff.1);
                    if !(w as u16 >= MIN_WIDTH) || !(h as u16 >= MIN_HEIGHT) {
                        self.is_resizing = false;
                        return Ok(());
                    }
                    match container.data_mut() {
                        crate::wm::container::ContainerType::Floating(c) => {
                            c.geometry.width = w as u16;
                            c.geometry.height = h as u16;
                            connection.configure_window(c.window_id(), &c.geometry.into())?;
                        }
                        _ => (),
                    }
                    self.is_resizing = false
                }

                _ => (),
            }
        }
        Ok(())
    }

    /// Handle a motion notify event.
    ///
    /// After checking which flag is active, we either move or resize the window.
    pub fn handle_motion_notify(
        &mut self,
        ev: &x11rb::protocol::xproto::MotionNotifyEvent,
    ) -> WmResult {
        let connection = self.connection();
        let dragging = self.is_dragging;
        let resizing = self.is_resizing;
        let workspace = self.workspace_for_window_mut(ev.event).ok_or_else(|| {
            Error::Generic(format!(
                "workspace error: unable to find workspace for window id {}",
                ev.event
            ))
        })?;

        let container = workspace.find_by_window_id_mut(ev.event)?;

        if !container.is_floating() {
            return Ok(());
        } else {
            if dragging {
                let last_event_position = container.last_position().unwrap();
                let diff = (
                    last_event_position.0 as i16 - ev.root_x,
                    last_event_position.1 as i16 - ev.root_y,
                );
                match container.data_mut() {
                    crate::wm::container::ContainerType::Floating(c) => {
                        c.geometry.x -= diff.0;
                        c.geometry.y -= diff.1;

                        connection.configure_window(c.window_id(), &c.geometry().into())?;
                    }
                    _ => (),
                }
                container.change_last_position((ev.root_x, ev.root_y))
            } else if resizing {
                let last_event_position = container.last_position().unwrap();
                let diff = (
                    last_event_position.0 as i16 - ev.root_x,
                    last_event_position.1 as i16 - ev.root_y,
                );
                let geom = container.data().geometry();
                let (w, h) = (geom.width as i16 - diff.0, geom.height as i16 - diff.1);
                if !(w as u16 >= MIN_WIDTH) || !(h as u16 >= MIN_HEIGHT) {
                    return Ok(());
                }
                match container.data_mut() {
                    crate::wm::container::ContainerType::Floating(c) => {
                        c.geometry.width = w as u16;
                        c.geometry.height = h as u16;
                        connection.configure_window(c.window_id(), &c.geometry.into())?;
                    }
                    _ => (),
                }
                container.change_last_position((ev.root_x - diff.0, ev.root_y - diff.1));
            }
        }

        Ok(())
    }

    /// Handle the execution of a given action.
    fn do_action(&mut self, action: Action) -> WmResult {
        match action {
            Action::Noop => {}
            Action::Kill => self.action_kill()?,
            Action::Goto(workspace) => self.action_goto(workspace as u32)?,
            Action::Move(workspace) => self.action_move(workspace as u32)?,
            Action::Execute(command) => self.action_execute(command)?,
            Action::Focus(direction) => self.action_focus(direction)?,
            Action::ChangeLayout(layout) => self.action_change_layout(layout)?,
            Action::CycleLayout => self.action_cycle_layout()?,
            Action::ToggleFloat => self.action_toggle_float()?,
        }

        Ok(())
    }

    fn action_execute(&mut self, command: String) -> WmResult {
        // TODO: get rid of this on release
        #[cfg(debug_assertions)]
        let process = std::process::Command::new("bash")
            .env("DISPLAY", ":1")
            .arg("-c")
            .args(
                command
                    .split(" ")
                    .map(|m| m.to_string())
                    .collect::<Vec<String>>(),
            )
            .spawn()?;

        #[allow(dead_code)]
        #[cfg(not(debug_assertions))]
        let process = std::process::Command::new("bash")
            .arg("-c")
            .args(
                command
                    .split(" ")
                    .map(|m| m.to_string())
                    .collect::<Vec<String>>(),
            )
            .spawn()?;

        #[cfg(debug_assertions)]
        println!("command: {command} has child process {}", process.id());

        Ok(())
    }

    fn action_kill(&mut self) -> WmResult {
        if let Some(window) = self.client_focus.focused_client() {
            if let Some(workspace) = self.workspace_for_window(window) {
                if let Ok(container) = workspace.find_by_window_id(window) {
                    if let Some(process_id) = container.data().process_id() {
                        if process_id != 0 {
                            let pid = format!("{process_id}");
                            std::process::Command::new("kill").arg(pid).spawn()?;
                            return Ok(());
                        }
                    }
                }
            }

            let process_id_reply = self
                .connection()
                .get_property(
                    false,
                    window,
                    *self.atoms.get("_NET_WM_PID").unwrap(),
                    AtomEnum::CARDINAL,
                    0,
                    1,
                )?
                .reply()?;

            if process_id_reply.value_len != 0 && process_id_reply.format == 32 {
                let process_id = process_id_reply.value32().unwrap().collect::<Vec<u32>>()[0];
                #[cfg(debug_assertions)]
                println!("killing {process_id}");
                std::process::Command::new("kill")
                    .arg(format!("{process_id}"))
                    .spawn()?;
                return Ok(());
            }

            self.connection().destroy_subwindows(window)?;
            self.connection().destroy_window(window)?;
        }

        Ok(())
    }

    fn action_focus(&mut self, direction: Direction) -> WmResult {
        if let Some(window) = self.client_focus.focused_client() {
            let workspace = self.get_focused_workspace()?;
            let container = workspace.find_by_window_id(window)?;
            let container_id = container.id();

            let container_to_focus_option = match direction {
                Direction::Right => Some(workspace.next_container(*container_id)),
                Direction::Left => Some(workspace.previous_container(*container_id)),
                Direction::Up => Some(workspace.previous_container(*container_id)),
                Direction::Down => Some(workspace.next_container(*container_id)),
            };

            if let Some(container_to_focus) = container_to_focus_option {
                if let Some(window_to_focus) = container_to_focus?.data().window_id() {
                    self.connection().set_input_focus(
                        InputFocus::NONE,
                        window_to_focus,
                        x11rb::CURRENT_TIME,
                    )?;
                    self.client_focus.set_focused_client(window_to_focus);
                }
            }
        }

        Ok(())
    }

    fn action_goto(&mut self, workspace_id: WorkspaceId) -> WmResult {
        let workspace = self.workspace_with_id(workspace_id).ok_or_else(|| {
            crate::errors::Error::Generic(format!(
                "workspace error: unable to find workspace with id {workspace_id}"
            ))
        })?;
        if let Some(current_workspace_id) = self.focused_workspace {
            if let Some(current_workspace) = self.workspace_with_id(current_workspace_id) {
                for each in current_workspace.iter_containers()? {
                    if let Some(wid) = each.data().window_id() {
                        self.connection().unmap_subwindows(wid)?;
                        self.connection().unmap_window(wid)?;
                    }
                }
            }
        }

        for container in workspace.iter_containers()? {
            if let Some(window) = container.data().window_id() {
                self.connection().map_window(window)?;
            }
        }

        self.focused_workspace = Some(workspace_id);

        Ok(())
    }

    fn action_move(&mut self, workspace_id: WorkspaceId) -> WmResult {
        // get currently focused client id, retrieve it from its workspace, find the other
        // workspace and move the client to that second workspace
        let connection = self.connection();
        let focused_client = self
            .client_focus
            .focused_client()
            .ok_or_else(|| Error::Generic("move error: no focused client".into()))?;
        let root_geometry = self.root_geometry()?;
        self.connection().unmap_subwindows(focused_client)?;
        self.connection().unmap_window(focused_client)?;
        let focused_workspace = self.get_focused_workspace_mut()?;
        let container = focused_workspace.remove_and_return_window(focused_client)?;
        focused_workspace.apply_layout(root_geometry, connection.clone())?;
        if let Some(other_workspace) = self.workspace_with_id_mut(workspace_id) {
            other_workspace.insert_container(container)?;
            other_workspace.apply_layout(root_geometry, connection)?;
        }

        Ok(())
    }

    fn action_change_layout(&mut self, layout: String) -> WmResult {
        let connection = self.connection();
        let root_geometry = self.root_geometry()?;
        let workspace = self.get_focused_workspace_mut()?;

        workspace.change_layout(layout)?;
        workspace.apply_layout(root_geometry, connection)?;

        Ok(())
    }
    fn action_cycle_layout(&mut self) -> WmResult {
        let connection = self.connection();
        let root_geometry = self.root_geometry()?;
        let workspace = self.get_focused_workspace_mut()?;

        workspace.cycle_layout()?;
        workspace.apply_layout(root_geometry, connection)?;
        Ok(())
    }

    fn action_toggle_float(&mut self) -> WmResult {
        let connection = self.connection();
        let root_geometry = self.root_geometry()?;
        let focused_client_id = match self.client_focus.focused_client() {
            Some(c) => c,
            None => return Err("clinet focus error: there is no client currently in focus.".into()),
        };
        let workspace = self.get_focused_workspace_mut()?;

        let container = workspace.find_by_window_id_mut(focused_client_id)?;

        if container.is_tiled() {
            container.into_floating()?
        } else {
            container.into_layout()?
        }

        let window_config = ConfigureWindowAux::new().stack_mode(Some(StackMode::ABOVE));
        connection
            .clone()
            .configure_window(focused_client_id, &window_config)?;
        workspace.apply_layout(root_geometry, connection.clone())?;
        connection.clone().flush()?;

        Ok(())
    }
}
