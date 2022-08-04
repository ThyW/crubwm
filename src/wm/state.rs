use cairo::{XCBConnection as CairoConnection, XCBDrawable, XCBSurface, XCBVisualType};
use x11::xlib::{Display, XOpenDisplay};
use x11rb::{
    connection::Connection,
    protocol::{
        randr::get_monitors,
        xproto::{
            ButtonIndex, ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt,
            CreateWindowAux, EventMask, FocusInEvent, GrabMode, InputFocus, KeyPressEvent,
            KeyReleaseEvent, Screen, StackMode, WindowClass,
        },
    },
    xcb_ffi::XCBConnection,
    CURRENT_TIME, NONE,
};

use crate::{
    config::{Config, Keybinds},
    errors::{Error, WmResult},
    ffi::find_xcb_visualtype,
    parsers::ConfigParser,
    wm::actions::{Action, Direction},
    wm::atoms::AtomManager,
    wm::bar::Bar,
    wm::container::{Client, ClientId, CT_MASK_TILING},
    wm::geometry::Geometry,
    wm::keyman::KeyManager,
    wm::layouts::LayoutMask,
    wm::monitors::Monitor,
    wm::workspace::Workspaces,
    wm::workspace::{Workspace, WorkspaceId},
};

use std::ffi::CStr;
use std::{collections::HashMap, rc::Rc};

use super::{atoms::AtomStruct, container::ContainerType, layouts::LayoutType};

pub struct State {
    connection: Rc<XCBConnection>,
    dpy: *mut Display,
    screen_index: usize,
    workspaces: Workspaces,
    focused_workspace: Option<WorkspaceId>,
    key_manager: KeyManager,
    last_client_id: ClientId,
    _atoms: HashMap<String, AtomStruct>,
    is_dragging: bool,
    is_resizing: bool,
    config: Rc<Config>,
    monitors: Vec<Monitor>,
    floating_modifier: u16,
    default_colormap: u32,
    xcb_connection: Rc<CairoConnection>,
    _cairo_visual: Rc<XCBVisualType>,
    bar_windows: Vec<u32>,
    bars: Vec<Bar>,
}

// Mask for any key
const ANY_KEY_MASK: u8 = 0;
// Mask for any mod key
const ANY_MOD_KEY_MASK: u16 = 32768;
// Window minimal width
const MIN_WIDTH: u16 = 160;
// Window minimal height
const MIN_HEIGHT: u16 = 90;
// Dragging speed
const DRAG_SPEED_COEFFICIENT: f32 = 1.5;

impl State {
    /// Connect to the X server and create WM state.
    ///
    /// If a name of the display is given, use that display, otherwise use the display from the
    /// DISPLAY environmental variable.
    pub fn new(name: Option<&str>, config: Rc<Config>) -> WmResult<Self> {
        let (connection, screen_index) =
            XCBConnection::connect(name.map(|s| unsafe { CStr::from_ptr(s.as_ptr() as _) }))?;
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

        let xcb_connection =
            unsafe { CairoConnection::from_raw_none(connection.get_raw_xcb_connection() as _) };
        let mut visual_ffi = find_xcb_visualtype(
            &connection,
            connection.setup().roots[screen_index].root_visual,
        )
        .unwrap();
        let visual = unsafe { XCBVisualType::from_raw_none(&mut visual_ffi as *mut _ as _) };

        // change root window attributes
        let change = ChangeWindowAttributesAux::default().event_mask(
            EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::STRUCTURE_NOTIFY
                | EventMask::PROPERTY_CHANGE,
        );

        let root_window = connection.setup().roots[screen_index as usize].root;
        let default_colormap = connection.setup().roots[screen_index as usize].default_colormap;
        connection.change_window_attributes(root_window, &change)?;
        connection.flush()?;

        let atoms = AtomManager::init_atoms(&connection)?;

        Ok(Self {
            connection: Rc::<XCBConnection>::new(connection),
            dpy: display,
            screen_index,
            workspaces: Vec::new(),
            focused_workspace: None,
            key_manager: KeyManager::default(),
            last_client_id: 0,
            _atoms: atoms,
            is_dragging: false,
            is_resizing: false,
            config,
            monitors: Vec::new(),
            floating_modifier: 64,
            default_colormap,
            xcb_connection: Rc::new(xcb_connection),
            _cairo_visual: Rc::new(visual),
            bar_windows: Vec::new(),
            bars: Vec::new(),
        })
    }

    /// Initiate the `KeyManager` with the Keybindings loaded in from a configuration file.
    pub fn init_keyman(&mut self, binds: Keybinds) -> WmResult {
        let dpy = self.display();
        self.key_manager.init(dpy, &binds)?;

        // ungrab any key with any modifier
        self.connection()
            .ungrab_key(ANY_KEY_MASK, self.root_window(), ANY_MOD_KEY_MASK)?;

        if let Some(mask) = self.key_manager.get_floating_modifier() {
            self.floating_modifier = mask;
        }

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

    /// Return the X id of the default colormap
    fn default_colormap(&self) -> u32 {
        self.default_colormap
    }

    /// Try and return the value of the `_NET_WM_NAME` or `WM_NAME` properties for the currently focused window.
    fn focused_window_name(&self) -> WmResult<String> {
        let mut ret_str: Option<String> = None;
        let ws = self.get_focused_workspace()?;
        if let Some(win) = ws.focus.focused_client() {
            if let Some(atom) = self._atoms.get("_NET_WM_NAME") {
                let ret = atom.get_property(win, self.connection())?;
                if let Some(first) = ret.first() {
                    let str: String = first.clone().try_into()?;
                    if !str.is_empty() && ret_str.is_none() {
                        ret_str = Some(str);
                    }
                }
            }
            if let Some(atom) = self._atoms.get("WM_NAME") {
                let ret = atom.get_property(win, self.connection())?;
                if let Some(first) = ret.first() {
                    let str: String = first.clone().try_into()?;
                    if !str.is_empty() && ret_str.is_none() {
                        ret_str = Some(str);
                    }
                }
            }
        }

        Ok(ret_str.unwrap_or_else(|| "N/A".to_string()))
    }

    /// Go through all workspaces, if they contain a given window: return the reference to the
    /// workspace, otherwise don't return anything.
    fn workspace_for_window(&self, wid: u32) -> Option<&Workspace> {
        self.workspaces
            .iter()
            .find(|workspace| workspace.contains_window(wid))
    }

    // Get a pointer to Xlib display structure. This method is used for handling keyboard
    // events(KeyPress and KeyRelease events).
    fn display(&mut self) -> *mut Display {
        self.dpy
    }

    /// Go through all workspaces, if they contain a given window: return a mutable reference to the
    /// workspace, otherwise don't return anything.
    fn workspace_for_window_mut(&mut self, wid: u32) -> Option<&mut Workspace> {
        self.workspaces
            .iter_mut()
            .find(|workspace| workspace.contains_window(wid))
    }

    /// Search for and return a reference to a workspace with the given workspace id.
    fn workspace_with_id<I: Into<WorkspaceId> + Copy>(&self, id: I) -> Option<&Workspace> {
        self.workspaces
            .iter()
            .find(|workspace| workspace.id == id.into())
    }

    /// Search for and return a reference to a workspace with the given workspace id.
    fn workspace_with_id_mut<I: Into<WorkspaceId> + Copy>(
        &mut self,
        id: I,
    ) -> Option<&mut Workspace> {
        self.workspaces
            .iter_mut()
            .find(|workspace| workspace.id == id.into())
    }

    /// Generate a new client identifier.
    fn new_client_id(&mut self) -> ClientId {
        self.last_client_id += 1;
        self.last_client_id
    }

    /// Get a referecnce to the underlying X connection.
    pub fn connection(&self) -> Rc<impl Connection> {
        self.connection.clone()
    }

    /// Handle the creation and initialisation of workspaces.
    ///
    /// In the future, this method should be loading workspace names, ids and indices from the
    /// Config structure.
    ///
    /// This method is also responsible for the creation and setup of monitors.
    pub fn init_workspaces(&mut self) -> WmResult {
        self.setup_monitors()?;
        for workspace_settings in self.config.workspace_settings.clone().into_iter() {
            let layout_mask = LayoutMask::from_slice(&workspace_settings.allowed_layouts)?;
            let (monitor_index, screen_size) =
                self.get_screen_size_for_workspace(workspace_settings.monitor.clone())?;
            self.workspaces.push(Workspace::new(
                workspace_settings.name.clone(),
                workspace_settings.identifier,
                layout_mask,
                self.root_window(),
                screen_size,
                self.monitors[monitor_index].id(),
            ));
            self.monitors[monitor_index].add_workspace(workspace_settings.identifier)
        }
        for monitor in self.monitors.iter_mut() {
            if let Err(e) = monitor.set_open_workspace(None) {
                eprintln!("{}", e)
            }
        }

        self.focus_workspace(self.workspaces[0].id, true)?;

        Ok(())
    }

    /// Helper function to determine which output id should go to which worksapce.
    fn get_screen_size_for_workspace(
        &self,
        monitor_number_string: String,
    ) -> WmResult<(usize, Geometry)> {
        // TODO: if this fails a warning should be returned.
        let monitor_number = monitor_number_string.parse::<usize>().unwrap_or(0);

        if let Some(monitor) = self.monitors.get(monitor_number) {
            return Ok((monitor_number, monitor.size()));
        }

        Err(format!("worksapce error: unable to construct workspace: monitor with index {monitor_number_string} not found.").into())
    }

    /// Create and setup monitors for workspaces.
    fn setup_monitors(&mut self) -> WmResult {
        let monitor_reply =
            get_monitors(self.connection().as_ref(), self.root_window(), false)?.reply()?;
        let mut current_monitor_id = 0u32;

        for monitor_info in monitor_reply.monitors {
            current_monitor_id += 1;
            let monitor = Monitor::from_monitor_info(monitor_info, current_monitor_id)?;
            self.monitors.push(monitor)
        }

        Ok(())
    }

    /// Create and setup status bar windows based on the status bar settings.
    pub fn setup_bars(&mut self) -> WmResult {
        let mut bars = Vec::new();
        // intitial bar construction
        for bar_settings in self.config.bar_settings.clone().into_iter() {
            bars.push(Bar::new(
                bar_settings.identifier,
                bar_settings.monitor,
                &bar_settings,
            )?);
        }
        // setup bars on different monitors
        for bar in bars.iter_mut() {
            let monitor = self
                .monitors
                .iter()
                .find(|monitor| monitor.id() == bar.monitor() + 1)
                .ok_or_else(|| {
                    Error::Generic(format!(
                        "Status bar error: No monitor with id {}.",
                        bar.monitor()
                    ))
                })?;
            let monitor_geometry = monitor.size();
            let bar_workspace_name_ids: Vec<(String, u32)> = self
                .config
                .workspace_settings
                .clone()
                .into_iter()
                .filter(|ws| ws.monitor.parse::<u32>().unwrap_or(0) == bar.monitor())
                .map(|ws| (ws.name, ws.identifier))
                .collect();
            // TODO
            // tell the bar what workspaces to display
            bar.create_workspaces(bar_workspace_name_ids);

            // initialize bar commands
            bar.update_widgets()?;

            // create bar windows and do all the necessary graphical setup
            //  - [x] setup a raw xcb connection
            //  - [x] find visual
            //  - [x] instantiate all the stuff
            //  - [x] create windows
            //  - [x] map window
            //  - [x] draw the segments

            let window_id = self.connection().generate_id()?;
            let screen = self.connection().setup().roots[self.screen_index].clone();
            let values = CreateWindowAux::new()
                .background_pixel(screen.black_pixel)
                .border_pixel(screen.black_pixel)
                .event_mask(
                    EventMask::STRUCTURE_NOTIFY | EventMask::EXPOSURE | EventMask::KEY_PRESS,
                );
            self.connection().create_window(
                screen.root_depth,
                window_id,
                screen.root,
                monitor_geometry.x,
                monitor_geometry.y,
                monitor_geometry.width,
                bar.settings()?.height as _, // this should be changed, it should be calculated from the bar font
                0,
                WindowClass::INPUT_OUTPUT,
                screen.root_visual,
                &values,
            )?;
            let mut visual_ffi = find_xcb_visualtype(
                self.connection.as_ref(),
                self.connection().setup().roots[self.screen_index].root_visual,
            )
            .unwrap();
            let visual = unsafe { XCBVisualType::from_raw_none(&mut visual_ffi as *mut _ as _) };

            let surface = XCBSurface::create(
                &self.xcb_connection,
                &XCBDrawable(window_id),
                &visual,
                monitor_geometry.width.into(),
                bar.settings()?.height.try_into().unwrap_or(15),
            )?;

            bar.set_surface(surface);
            bar.set_window_id(window_id);
            if let Ok(h) = bar.get_height() {
                self.connection().configure_window(
                    window_id,
                    &ConfigureWindowAux::new()
                        .height(h)
                        .stack_mode(StackMode::ABOVE),
                )?;
                let surface = XCBSurface::create(
                    &self.xcb_connection,
                    &XCBDrawable(window_id),
                    &visual,
                    monitor_geometry.width as _,
                    h as _,
                )?;
                bar.set_surface(surface);
                let mut geom = monitor_geometry;
                geom.height = h as _;
                bar.set_geometry(geom);
                let connection = self.connection();
                for workspace in self.workspaces.iter_mut() {
                    if workspace.monitor == bar.monitor() + 1 {
                        let g = workspace.screen() - geom;
                        workspace.set_screen(g);
                        workspace.apply_layout(connection.clone(), None, self.default_colormap)?;
                    }
                }
            }
            self.bar_windows.push(window_id);
            self.connection().map_window(window_id)?;
            self.connection().flush()?;
        }

        self.bars = bars;

        Ok(())
    }

    /// Update and redraw all bar windows.
    pub fn update_bars(&mut self) -> WmResult {
        let window_name = self
            .focused_window_name()
            .unwrap_or_else(|_| "NAN".to_string());
        for bar in self.bars.iter_mut() {
            let monitors: Vec<&Monitor> = self
                .monitors
                .iter()
                .filter(|mon| mon.id() == bar.monitor() + 1)
                .collect();
            if let Some(monitor) = monitors.first() {
                if let Ok(ws) = monitor.get_open_workspace() {
                    bar.update(self.focused_workspace, Some(ws), window_name.clone())?
                } else {
                    bar.update(self.focused_workspace, None, window_name.clone())?
                }
            }
            bar.redraw()?
        }

        Ok(())
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

    /// Return a mutable reference to the currently open workspace on a monitor which the cursor is
    /// on.
    fn get_workspace_under_cursor_mut(&mut self) -> WmResult<&mut Workspace> {
        let cursor = self
            .connection()
            .query_pointer(self.root_window())?
            .reply()?;
        let (x, y) = (cursor.root_x, cursor.root_y);
        let mons = self.monitors.clone();
        let mut id = None;

        for monitor in mons.iter() {
            let g = monitor.size();
            if (x >= g.x)
                && (x <= g.x + g.width as i16)
                && (y >= g.y)
                && (y <= g.y + g.height as i16)
            {
                id = Some(monitor.get_open_workspace()?);
                break;
            }
        }

        if let Some(id) = id {
            if let Some(ws) = self.workspace_with_id_mut(id) {
                return Ok(ws);
            }
        }

        Err("cursor is not in any workspace currently!".into())
    }

    /// Focus a workspace.
    pub fn focus_workspace(&mut self, workspace_id: WorkspaceId, warp_pointer: bool) -> WmResult {
        // 1. find the currently focused monitor and workspace
        //
        // 2. find the monitor of the 'to be' focused workspace
        //
        // 3.A. if the two workspaces are on the same monitor, unmap one, map the other, set the
        //      newly focused workspace as the monitor's open workspace.
        // 3.B. if the two workspaces are not on the same monitor:
        //      3.B.1 if the 'to be' focused workspace is open on the other monitor, do nothing and
        //            just focus the next workspace and set the monitor to be focused.
        //      3.B.2 if the 'to be' focused workspace is not open on the other monitor, unmap
        //            the old workspace, map the new one and set give the monitor the focus.
        if workspace_id == self.focused_workspace.unwrap_or(0) {
            return Ok(());
        }
        let current_focused_monitor_id = self.get_focused_or_first_monitor()?.id();
        let new_focused_monitor_id = self.monitor_for_workspace_mut(workspace_id)?.id();

        // variant A
        if current_focused_monitor_id == new_focused_monitor_id {
            if let Ok(focused_workspace) = self.get_focused_workspace() {
                for container in focused_workspace.iter_containers()? {
                    if let Some(wid) = container.data().window_id() {
                        self.connection().unmap_subwindows(wid)?;
                        self.connection().unmap_window(wid)?;
                    }
                }
            }

            let workspace = self
                .workspace_with_id(workspace_id)
                .ok_or_else(|| Error::Generic("No workspace with given id exists".into()))?;

            for container in workspace.iter_containers()? {
                if let Some(wid) = container.data().window_id() {
                    self.connection().map_window(wid)?;
                    self.connection().map_subwindows(wid)?;
                }
            }

            self.monitor_with_id_mut(current_focused_monitor_id)?
                .set_open_workspace(Some(workspace_id))?;
            self.monitor_with_id_mut(current_focused_monitor_id)?
                .focus(true);
            self.focused_workspace = Some(workspace_id);
        } else {
            // variant B
            let open_workspace_id = self
                .monitor_with_id_mut(new_focused_monitor_id)?
                .get_open_workspace()?;

            // variant B.1.
            if open_workspace_id == workspace_id {
                self.monitor_with_id_mut(new_focused_monitor_id)?
                    .focus(true);
                self.monitor_with_id_mut(current_focused_monitor_id)?
                    .focus(false);
                self.focused_workspace = Some(workspace_id);
            } else {
                // variant B.2.
                if let Some(focused_workspace) = self.workspace_with_id(open_workspace_id) {
                    for container in focused_workspace.iter_containers()? {
                        if let Some(wid) = container.data().window_id() {
                            self.connection().unmap_subwindows(wid)?;
                            self.connection().unmap_window(wid)?;
                        }
                    }
                }

                let workspace = self
                    .workspace_with_id(workspace_id)
                    .ok_or_else(|| Error::Generic("No workspace with given id exists".into()))?;

                for container in workspace.iter_containers()? {
                    if let Some(wid) = container.data().window_id() {
                        self.connection().map_window(wid)?;
                        self.connection().map_subwindows(wid)?;
                    }
                }
                let monitor = self.monitor_with_id_mut(new_focused_monitor_id)?;
                monitor.focus(true);
                monitor.set_open_workspace(Some(workspace_id))?;
                self.monitor_with_id_mut(current_focused_monitor_id)?
                    .focus(false);
                self.focused_workspace = Some(workspace_id);
            }
        }

        let workspace = self.get_focused_workspace()?;
        let size = workspace.screen();

        if warp_pointer {
            self.connection().warp_pointer(
                NONE,
                self.root_window(),
                0,
                0,
                0,
                0,
                size.x + (size.width / 2) as i16,
                size.y + (size.height / 2) as i16,
            )?;
        }

        Ok(())
    }

    /// If there is a focused monitor, return a reference to it, otherwsie return a reference
    /// to the first monitor.
    fn get_focused_or_first_monitor(&self) -> WmResult<&Monitor> {
        if let Ok(monitor) = self.get_focused_monitor() {
            return Ok(monitor);
        } else if let Some(monitor) = self.monitors.get(0) {
            return Ok(monitor);
        }

        Err("There are currently no monitors available for this X display.".into())
    }

    /// Return a reference to the currently focused monitor.
    fn get_focused_monitor(&self) -> WmResult<&Monitor> {
        for monitor in self.monitors.iter() {
            if monitor.is_focused() {
                return Ok(monitor);
            }
        }

        Err("There are no currently focused monitors.".into())
    }

    /// Retrun a mutable reference to the monitor which the workspace with the given id is
    /// currently on.
    fn monitor_for_workspace_mut(&mut self, workspace_id: WorkspaceId) -> WmResult<&mut Monitor> {
        for monitor in self.monitors.iter_mut() {
            if monitor.contains(&workspace_id) {
                return Ok(monitor);
            }
        }

        Err("Workspace is not located in any monitor.".into())
    }

    /// Retrun an immutable reference to the monitor which the workspace with the given id is
    /// currently on.
    #[allow(unused)]
    fn monitor_for_workspace(&self, workspace_id: WorkspaceId) -> WmResult<&Monitor> {
        for monitor in self.monitors.iter() {
            if monitor.contains(&workspace_id) {
                return Ok(monitor);
            }
        }

        Err("Workspace is not located in any monitor.".into())
    }

    /// Return a mutable reference to the monitor given its id.
    fn monitor_with_id_mut(&mut self, id: u32) -> WmResult<&mut Monitor> {
        for monitor in self.monitors.iter_mut() {
            if monitor.id() == id {
                return Ok(monitor);
            }
        }

        Err(format!("No monitor with id {} found.", id).into())
    }

    /// Become a window manager, take control of all open windows on the X server.
    pub fn become_wm(&mut self) -> WmResult {
        let connection = self.connection();
        let root_window = self.root_window();
        let query_tree_cookie = connection.query_tree(root_window)?;
        let query_tree_reply = query_tree_cookie.reply()?;

        for window_id in query_tree_reply.children {
            self.manage_window(window_id)?
        }

        Ok(())
    }

    /// Let a window be managed by the window manager.
    pub fn manage_window(&mut self, window: u32) -> WmResult {
        let config = self.config.clone();
        let connection = self.connection();
        let geometry = self.connection().get_geometry(window)?.reply()?;
        let new_client_id = self.new_client_id();
        let default_colormap = self.default_colormap();
        let bar_windows = self.bar_windows.clone();

        if bar_windows.contains(&window) {
            return Ok(());
        }

        let workspace = self.get_workspace_under_cursor_mut()?;
        let id = workspace.id;
        self.focus_workspace(id, false)?;

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
            self.floating_modifier,
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
            self.floating_modifier,
        )?;

        self.connection()
            .reparent_window(window, self.root_window(), 0, 0)?;

        self.connection().map_window(window)?;
        self.get_focused_workspace_mut()?
            .apply_layout(connection, None, default_colormap)?;

        self.connection()
            .set_input_focus(InputFocus::PARENT, window, CURRENT_TIME)?;
        self.get_focused_workspace_mut()?
            .focus
            .set_focused_client(window);

        Ok(())
    }

    /// This method is called when a window is destroyed.
    ///
    /// First, start by finding the window than remove it and apply the correct geometries to the
    /// rest of the windows in the workspace.
    pub fn unmanage_window(&mut self, window: u32) -> WmResult {
        let connection = self.connection();
        let default_colormap = self.default_colormap();

        let workspace_option = self.workspace_for_window_mut(window);
        let mut workspace_id = None;

        if let Some(workspace) = workspace_option {
            workspace.remove_window(window)?;
            let _ = workspace_id.insert(workspace.id);
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
        if let Some(workspace_id) = workspace_id {
            if let Some(workspace) = self.workspace_with_id_mut(workspace_id) {
                workspace.apply_layout(connection, None, default_colormap)?
            }
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

        self.focus_workspace(id, false)?;
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
        let default_colormap = self.default_colormap();
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
                    if let crate::wm::container::ContainerType::Floating(c) = container.data_mut() {
                        c.geometry.x -= diff.0;
                        c.geometry.y -= diff.1;
                        c.draw_borders(connection, default_colormap)?;
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
                    if (w as u16) < MIN_WIDTH || (h as u16) < MIN_HEIGHT {
                        self.is_resizing = false;
                        return Ok(());
                    }
                    if let crate::wm::container::ContainerType::Floating(c) = container.data_mut() {
                        c.geometry.width = w as u16;
                        c.geometry.height = h as u16;
                        c.draw_borders(connection, default_colormap)?;
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
        let default_colormap = self.default_colormap();
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
        } else if dragging {
            let last_event_position = container.last_position().unwrap();
            let diff = (
                (last_event_position.0 as i16 - ev.root_x) as f32 * DRAG_SPEED_COEFFICIENT,
                (last_event_position.1 as i16 - ev.root_y) as f32 * DRAG_SPEED_COEFFICIENT,
            );
            if let crate::wm::container::ContainerType::Floating(c) = container.data_mut() {
                c.geometry.x -= diff.0 as i16;
                c.geometry.y -= diff.1 as i16;

                c.draw_borders(connection, default_colormap)?;
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
            if (w as u16) < MIN_WIDTH || (h as u16) < MIN_HEIGHT {
                return Ok(());
            }
            if let crate::wm::container::ContainerType::Floating(c) = container.data_mut() {
                c.geometry.width = w as u16;
                c.geometry.height = h as u16;
                c.draw_borders(connection, default_colormap)?;
            }
            container.change_last_position((ev.root_x - diff.0 as i16, ev.root_y - diff.1 as i16));
        }

        Ok(())
    }

    pub fn handle_focus_in(&mut self, ev: &FocusInEvent) -> WmResult {
        let connection = self.connection();
        if let Some(workspace) = self.workspace_for_window_mut(ev.event) {
            if let Some(focused) = workspace.focus.focused_client() {
                connection.set_input_focus(InputFocus::PARENT, focused, CURRENT_TIME)?;
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
            Action::Swap(direction) => self.action_swap(direction)?,
            Action::ReloadConfig => self.action_reload_config()?,
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
                    .split(' ')
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
            if let Some(pid_atom) = self._atoms.get("_NET_WM_PID") {
                let pid: u32 = pid_atom.get_property(window, self.connection())?[0]
                    .clone()
                    .try_into()?;
                let _ = std::process::Command::new("kill")
                    .arg(format!("{pid}"))
                    .status()?;
            }
        }

        Ok(())
    }

    fn action_focus(&mut self, direction: Direction) -> WmResult {
        let connection = self.connection();
        let default_colormap = self.default_colormap();
        if let Some(window) = self.get_focused_workspace_mut()?.focus.focused_client() {
            let workspace = self.get_focused_workspace_mut()?;
            let container = workspace.find_by_window_id(window)?;
            let container_id = container.id();
            let layout = *workspace.current_layout();

            let container_to_focus_option = match direction {
                Direction::Next => Some(workspace.next_container(*container_id)),
                Direction::Previous => Some(workspace.previous_container(*container_id)),
            };

            if let Some(container_to_focus) = container_to_focus_option {
                let container = container_to_focus?.data();
                let size = container.geometry();
                if let Some(window_to_focus) = container.window_id() {
                    workspace.focus.set_focused_client(window_to_focus);
                    if matches!(layout, LayoutType::StackingHorizontal) {
                        workspace.apply_layout(connection.clone(), None, default_colormap)?;
                    }
                    connection.set_input_focus(
                        InputFocus::PARENT,
                        window_to_focus,
                        x11rb::CURRENT_TIME,
                    )?;
                    self.connection().warp_pointer(
                        NONE,
                        self.root_window(),
                        0,
                        0,
                        0,
                        0,
                        size.x + (size.width / 2) as i16,
                        size.y + (size.height / 2) as i16,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn action_goto(&mut self, workspace_id: WorkspaceId) -> WmResult {
        self.focus_workspace(workspace_id, true)?;

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
        let workspace_id = self
            .workspace_with_id(workspace_id)
            .ok_or_else(|| {
                Error::Generic(format!(
                    "move error: no workspace with id {workspace_id} found"
                ))
            })?
            .id;
        let default_colormap = self.default_colormap();

        self.connection().unmap_subwindows(focused_client)?;
        self.connection().unmap_window(focused_client)?;

        let focused_workspace = self.get_focused_workspace_mut()?;
        let container = focused_workspace.remove_and_return_window(focused_client)?;
        self.get_focused_workspace_mut()?.apply_layout(
            connection.clone(),
            None,
            default_colormap,
        )?;

        let other_workspace = self.workspace_with_id_mut(workspace_id).unwrap();
        other_workspace.insert_container(container)?;
        other_workspace.apply_layout(connection, None, default_colormap)?;

        let monitor = self.monitor_for_workspace_mut(workspace_id)?;
        if monitor.get_open_workspace()? == workspace_id {
            self.connection().map_window(focused_client)?;
            self.connection().map_subwindows(focused_client)?;
        }

        Ok(())
    }

    fn action_change_layout(&mut self, layout: String) -> WmResult {
        let connection = self.connection();
        let default_colormap = self.default_colormap();

        self.get_focused_workspace_mut()?.change_layout(layout)?;
        self.get_focused_workspace_mut()?
            .apply_layout(connection, None, default_colormap)?;

        Ok(())
    }

    fn action_cycle_layout(&mut self) -> WmResult {
        let default_colormap = self.default_colormap();
        let connection = self.connection();
        let workspace = self.get_focused_workspace_mut()?;

        workspace.cycle_layout()?;
        workspace.apply_layout(connection, None, default_colormap)?;
        Ok(())
    }

    fn action_toggle_float(&mut self) -> WmResult {
        let connection = self.connection();
        let focused_client_id = match self.get_focused_workspace_mut()?.focus.focused_client() {
            Some(c) => c,
            None => return Err("clinet focus error: there is no client currently in focus.".into()),
        };
        let default_colormap = self.default_colormap();
        let workspace = self.get_focused_workspace_mut()?;

        let container = workspace.find_by_window_id_mut(focused_client_id)?;

        if container.is_in_layout() {
            container.change_to_floating()?
        } else {
            container.change_to_layout()?
        }

        let window_config = ConfigureWindowAux::new().stack_mode(Some(StackMode::ABOVE));
        connection.configure_window(focused_client_id, &window_config)?;
        workspace.apply_layout(connection.clone(), None, default_colormap)?;
        connection.set_input_focus(InputFocus::PARENT, focused_client_id, CURRENT_TIME)?;
        connection.flush()?;

        Ok(())
    }

    fn action_swap(&mut self, direction: Direction) -> WmResult {
        let connection = self.connection();
        let default_colormap = self.default_colormap();
        if let Some(window) = self.get_focused_workspace_mut()?.focus.focused_client() {
            let workspace = self.get_focused_workspace_mut()?;
            let container = workspace.find_by_window_id(window)?;
            let container_id = container.id();

            let container_to_focus_option = match direction {
                Direction::Next => Some(workspace.next_container(*container_id)),
                Direction::Previous => Some(workspace.previous_container(*container_id)),
            };

            if let Some(container_to_focus) = container_to_focus_option {
                let swap_with = container_to_focus?.id();
                workspace.swap(*container_id, *swap_with)?;
                workspace.apply_layout(connection, None, default_colormap)?;
            }
        }

        Ok(())
    }

    fn action_reload_config(&mut self) -> WmResult {
        // TODO: Take a look at how monitor changes should be handled
        let path = &self.config.path;
        let config = ConfigParser::parse_with_path(path)?;
        let mask: u32 =
            (EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::BUTTON_MOTION).into();
        self.config = Rc::new(config);
        let connection = self.connection();
        let root_window = self.root_window();
        let screen_geom = self.root_geometry()?;

        // redo keybinds
        self.init_keyman(self.config.keybinds.clone())?;
        // regrab keys for all clients, reapply client attributes and reapply layouts
        for workspace in self.workspaces.iter_mut() {
            workspace.set_screen(screen_geom);
            for container in workspace.containers_mut().iter_mut() {
                if let Some(window_id) = container.data().window_id() {
                    connection.ungrab_button(ButtonIndex::ANY, window_id, ANY_MOD_KEY_MASK)?;
                    connection.grab_button(
                        true,
                        window_id,
                        mask as u16,
                        GrabMode::ASYNC,
                        GrabMode::ASYNC,
                        root_window,
                        NONE,
                        ButtonIndex::M1,
                        self.floating_modifier,
                    )?;

                    connection.grab_button(
                        true,
                        window_id,
                        mask as u16,
                        GrabMode::ASYNC,
                        GrabMode::ASYNC,
                        root_window,
                        NONE,
                        ButtonIndex::M3,
                        self.floating_modifier,
                    )?;
                }
                match container.data_mut() {
                    ContainerType::InLayout(client) => client.change_config(&self.config),
                    ContainerType::Floating(client) => client.change_config(&self.config),

                    _ => (),
                }
            }

            workspace.apply_layout(connection.clone(), None, self.default_colormap)?;
        }

        // reapply bar settings
        for bar_window in self.bar_windows.iter() {
            connection.destroy_window(*bar_window)?;
        }

        self.bar_windows.clear();
        self.setup_bars()?;

        Ok(())
    }
}
