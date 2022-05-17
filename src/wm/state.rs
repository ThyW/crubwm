use x11::xlib::{Display, XOpenDisplay};
use x11rb::{
    connect,
    connection::Connection,
    protocol::{
        randr::get_monitors,
        xproto::{
            AtomEnum, ButtonIndex, ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt,
            EventMask, GrabMode, InputFocus, KeyPressEvent, KeyReleaseEvent, Screen, StackMode,
        },
    },
    rust_connection::RustConnection,
    CURRENT_TIME, NONE,
};

use crate::{
    config::{Config, Keybinds},
    errors::{Error, WmResult},
    wm::actions::{Action, Direction},
    wm::atoms::AtomManager,
    wm::container::{Client, ClientId, CT_MASK_TILING},
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
    key_manager: KeyManager,
    last_client_id: ClientId,
    atoms: HashMap<String, u32>,
    is_dragging: bool,
    is_resizing: bool,
    config: Rc<Config>,
}

const ANY_KEY_MASK: u8 = 0;
const ANY_MOD_KEY_MASK: u16 = 32768;
const MIN_WIDTH: u16 = 160;
const MIN_HEIGHT: u16 = 90;
// TODO: make this a config setting
const DRAG_SPEED_COEFFICIENT: f32 = 1.5;

impl State {
    /// Connect to the X server and create WM state.
    ///
    /// If a name of the display is given, use that display, otherwise use the display from the
    /// DISPLAY environmental variable.
    pub fn new(name: Option<&str>, config: Rc<Config>) -> WmResult<Self> {
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
            connection: Rc::<RustConnection>::new(connection),
            dpy: display,
            screen_index,
            workspaces: Vec::new(),
            focused_workspace: None,
            key_manager: KeyManager::default(),
            last_client_id: 0,
            atoms,
            is_dragging: false,
            is_resizing: false,
            config,
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

    /// Return the size of the root window.
    #[allow(unused)]
    fn root_geometry(&self) -> WmResult<Geometry> {
        let geom = self
            .connection()
            .get_geometry(self.root_window())?
            .reply()?
            .into();

        Ok(geom)
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
    pub fn init_workspaces(&mut self) -> WmResult {
        for workspace_settings in self.config.workspace_settings.clone().into_iter() {
            let layout_mask = LayoutMask::from_slice(&workspace_settings.allowed_layouts)?;
            let screen_size =
                self.get_screen_size_for_workspace(workspace_settings.monitor.clone())?;
            println!("{}", screen_size);
            self.workspaces.push(Workspace::new(
                workspace_settings.name.clone(),
                workspace_settings.identifier,
                layout_mask,
                self.root_window(),
                screen_size,
            ));
        }
        println!("here");
        self.focus_workspace(self.workspaces[0].id)?;

        Ok(())
    }

    /// Helper function to determine which output id should go to witch worksapce.
    fn get_screen_size_for_workspace(&self, monitor_number_string: String) -> WmResult<Geometry> {
        // TODO: if this fails a warning should be returned.
        let monitor_number = monitor_number_string.parse::<usize>().unwrap_or(0);

        let monitors = get_monitors(self.connection().as_ref(), self.root_window(), false)?
            .reply()?
            .monitors
            .clone();

        if let Some(monitor) = monitors.get(monitor_number) {
            let mut geometry = Geometry::default();
            geometry.x = monitor.x;
            geometry.y = monitor.y;
            geometry.width = monitor.width;
            geometry.height = monitor.height;
            return Ok(geometry);
        }

        Err(format!("worksapce error: unable to construct workspace: monitor with index {monitor_number_string} not found.").into())
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

    #[allow(unused)]
    fn get_workspace_under_cursor(&self) -> WmResult<&Workspace> {
        let cursor = self
            .connection()
            .query_pointer(self.root_window())?
            .reply()?;
        let (x, y) = (cursor.root_x, cursor.root_y);

        for ws in self.workspaces.iter() {
            let g = ws.screen();
            if x >= g.x && x <= g.x + g.width as i16 && y >= g.y && y <= g.y + g.height as i16 {
                return Ok(ws);
            }
        }

        Err("cursor is not in any workspace currently!".into())
    }

    fn get_workspace_under_cursor_mut(&mut self) -> WmResult<&mut Workspace> {
        let cursor = self
            .connection()
            .query_pointer(self.root_window())?
            .reply()?;
        let (x, y) = (cursor.root_x, cursor.root_y);

        for ws in self.workspaces.iter_mut() {
            let g = ws.screen();
            if x >= g.x && x <= g.x + g.width as i16 && y >= g.y && y <= g.y + g.height as i16 {
                return Ok(ws);
            }
        }

        Err("cursor is not in any workspace currently!".into())
    }

    pub fn focus_workspace(&mut self, workspace_id: WorkspaceId) -> WmResult {
        if workspace_id == self.focused_workspace.unwrap_or(0) {
            return Ok(());
        }
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
        let config = self.config.clone();
        let connection = self.connection();
        let geometry = self.connection().get_geometry(window)?.reply()?;
        let new_client_id = self.new_client_id();

        let workspace = self.get_workspace_under_cursor_mut()?;
        let id = workspace.id;
        self.focus_workspace(id)?;

        self.get_focused_workspace_mut()?.insert_client(
            Client::new_without_process_id(window, geometry, new_client_id, &config),
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

        self.connection()
            .ungrab_button(ButtonIndex::ANY, window, ANY_MOD_KEY_MASK)?;

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
            .apply_layout(connection, None)?;

        self.connection()
            .set_input_focus(InputFocus::NONE, window, CURRENT_TIME)?;
        self.get_focused_workspace_mut()?
            .focus
            .set_focused_client(window);

        Ok(())
    }

    /// Let multiple windows be managed by the window manager.
    ///
    /// For performance sake, this method does not call `manage_window` internally.
    pub fn manage_windows(&mut self, windows_and_geometries: Vec<(u32, Geometry)>) -> WmResult {
        let connection = self.connection();
        let config = self.config.clone();
        let new_client_ids = (0usize..windows_and_geometries.len())
            .map(|_| self.new_client_id())
            .collect::<Vec<u64>>();
        self.get_focused_workspace_mut()?.insert_many(
            windows_and_geometries
                .iter()
                .enumerate()
                .map(|(index, (window, geometry))| {
                    Client::new_without_process_id(
                        *window,
                        *geometry,
                        new_client_ids[index],
                        &config,
                    )
                })
                .collect::<Vec<Client>>()
                .into_iter(),
            windows_and_geometries
                .iter()
                .map(|_| CT_MASK_TILING)
                .collect::<Vec<u8>>()
                .into_iter(),
        );
        self.get_focused_workspace_mut()?
            .apply_layout(connection, None)?;

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
        let connection = self.connection();

        let workspace_option = self.workspace_for_window_mut(window);

        if let Some(workspace) = workspace_option {
            workspace.remove_window(window)?;
            workspace.apply_layout(connection, None)?;
        }

        // set input focus to previously focused client
        if let Some(previous_window_id) = self
            .get_focused_workspace_mut()?
            .focus
            .previously_focused_client()
        {
            self.connection().set_input_focus(
                InputFocus::NONE,
                previous_window_id,
                CURRENT_TIME,
            )?;
            self.get_focused_workspace_mut()?
                .focus
                .remove_client(window);
        }

        Ok(())
    }

    /// Handle an enter window event.
    ///
    /// This method is responsible for switching input focus to the newly entered window.
    /// In the future, this will also handle the decorators, WM properties and other necessary
    /// things.
    pub fn handle_enter_event(&mut self, window: u32) -> WmResult {
        let mut id = self.get_focused_workspace()?.id;

        if let Some(workspace) = self.workspace_for_window(window) {
            id = workspace.id
        }

        self.focus_workspace(id)?;
        self.get_focused_workspace_mut()?
            .focus
            .set_focused_client(window);

        self.connection()
            .set_input_focus(InputFocus::NONE, window, CURRENT_TIME)?;

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
                    (last_event_position.0 as i16 - ev.root_x) as f32 * DRAG_SPEED_COEFFICIENT,
                    (last_event_position.1 as i16 - ev.root_y) as f32 * DRAG_SPEED_COEFFICIENT,
                );
                match container.data_mut() {
                    crate::wm::container::ContainerType::Floating(c) => {
                        c.geometry.x -= diff.0 as i16;
                        c.geometry.y -= diff.1 as i16;

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

        #[cfg(not(debug_assertions))]
        let _ = std::process::Command::new("bash")
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
        if let Some(window) = self.get_focused_workspace_mut()?.focus.focused_client() {
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
        let connection = self.connection();
        if let Some(window) = self.get_focused_workspace_mut()?.focus.focused_client() {
            let workspace = self.get_focused_workspace_mut()?;
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
                    connection.set_input_focus(
                        InputFocus::NONE,
                        window_to_focus,
                        x11rb::CURRENT_TIME,
                    )?;
                    workspace.focus.set_focused_client(window_to_focus);
                }
            }
        }

        Ok(())
    }

    fn action_goto(&mut self, workspace_id: WorkspaceId) -> WmResult {
        self.focus_workspace(workspace_id)?;

        Ok(())
    }

    fn action_move(&mut self, workspace_id: WorkspaceId) -> WmResult {
        // get currently focused client id, retrieve it from its workspace, find the other
        // workspace and move the client to that second workspace
        let connection = self.connection();
        let focused_client = self
            .get_focused_workspace_mut()?
            .focus
            .focused_client()
            .ok_or_else(|| Error::Generic("move error: no focused client".into()))?;

        self.connection().unmap_subwindows(focused_client)?;
        self.connection().unmap_window(focused_client)?;

        let focused_workspace = self.get_focused_workspace_mut()?;
        let container = focused_workspace.remove_and_return_window(focused_client)?;
        self.get_focused_workspace_mut()?
            .apply_layout(connection.clone(), None)?;

        if let Some(other_workspace) = self.workspace_with_id_mut(workspace_id) {
            other_workspace.insert_container(container)?;
            other_workspace.apply_layout(connection, None)?;
        }

        Ok(())
    }

    fn action_change_layout(&mut self, layout: String) -> WmResult {
        let connection = self.connection();

        self.get_focused_workspace_mut()?.change_layout(layout)?;
        self.get_focused_workspace_mut()?
            .apply_layout(connection, None)?;

        Ok(())
    }

    fn action_cycle_layout(&mut self) -> WmResult {
        let connection = self.connection();
        let workspace = self.get_focused_workspace_mut()?;

        workspace.cycle_layout()?;
        workspace.apply_layout(connection, None)?;
        Ok(())
    }

    fn action_toggle_float(&mut self) -> WmResult {
        let connection = self.connection();
        let focused_client_id = match self.get_focused_workspace_mut()?.focus.focused_client() {
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
        workspace.apply_layout(connection.clone(), None)?;
        connection
            .clone()
            .set_input_focus(InputFocus::PARENT, focused_client_id, CURRENT_TIME)?;
        connection.flush()?;

        Ok(())
    }
}
