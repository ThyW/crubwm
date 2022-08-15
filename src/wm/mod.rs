use std::{rc::Rc, thread::spawn};

use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{AtomEnum, ConnectionExt, PropMode},
        Event,
    },
};

use crate::{
    config::Config,
    errors::WmResult,
    log::{log, LL_ALL, LL_NORMAL},
    parsers::{Command, CommandType},
    wm::state::State,
};

pub mod actions;
pub mod atoms;
pub mod bar;
pub mod container;
pub mod focus_stack;
pub mod geometry;
pub mod keyman;
pub mod layouts;
pub mod monitors;
pub mod state;
pub mod workspace;

fn print_help_message() {
    println!("crubwm is a tiling X window manager.\n");
    println!("Here is a list of all command line options:\n");
    println!("  -h or --help   \t\tPrint this message.");
    println!("  --config [PATH]\t\tUse a different config file.");
}

/// The WM struct, holding all the necessary state and information for and about the operation of
/// the window manager.
pub struct Wm {
    /// The configuration of the window manager, holding all keybinds, hooks and settings.
    config: Rc<crate::config::Config>,
    /// Window manager's state. Holds information about X server connection, clients, workspaces
    /// geometries, etc...
    state: State,
}

impl Wm {
    /// Create a new window manager instance.
    pub fn new(config: Config) -> WmResult<Self> {
        let c = config.clone();
        let display_name = match c.settings.display_name.is_empty() {
            false => Some(c.settings.display_name.as_str()),
            true => None,
        };

        let config = Rc::new(config);

        // create the state manager here.
        let state = State::new(display_name, config.clone())?;

        Ok(Self { config, state })
    }

    /// Run the window manager, this instantiates the event loop, constructs workspaces and does
    /// all the necessary work in order for the window manager to function.
    pub fn run(&mut self, commands: Vec<Command>) -> WmResult {
        for command in commands {
            if command.get_type() == &CommandType::Help {
                print_help_message();
                return Ok(());
            }
        }

        log("Logger initialized.", LL_NORMAL);

        // run startup hooks
        self.config.start_hooks.run()?;
        // instantiate workspaces
        self.state.init_workspaces()?;
        // after setting up monitors and workspaces, setup up status bar
        self.state.setup_bars()?;
        // check for all open windows and manage them
        // self.state.become_wm()?;
        // notify the window manager of the keybinds
        self.state.init_keyman(self.config.keybinds.clone())?;
        // run the hooks after creating wm

        // run the bar update thread
        let bar_windows = self.state.bar_windows();
        let conn = self.state.connection();

        let bar_atom = self
            .state
            .connection()
            .intern_atom(false, b"__BAR_UPDATE")?
            .reply()?
            .atom;
        let _ = spawn(move || {
            let mut last_time = std::time::Instant::now();
            let mut switch = 0;
            loop {
                if last_time.elapsed().as_secs() >= 1 {
                    last_time = std::time::Instant::now();
                    for win in bar_windows.iter() {
                        log("changing property for bar window.", LL_NORMAL);
                        if conn
                            .change_property(
                                PropMode::REPLACE,
                                *win,
                                bar_atom,
                                AtomEnum::INTEGER,
                                8,
                                1,
                                &[switch],
                            )
                            .is_ok()
                        {}
                    }
                    switch = if switch.eq(&1) { 0 } else { 1 };
                    conn.flush().unwrap();
                }
            }
        });

        self.state.update_bars()?;

        let mut first = false;
        let mut ran = false;

        // run the event loop, don't stop on errors, just report them and keep going.
        loop {
            if !first {
                first = true;
            } else if !ran {
                self.config.start_hooks.run_after()?;
                ran = true;
            }
            self.state.connection().flush()?;
            self.state.update_bars()?;
            let event = self.state.connection().wait_for_event()?;

            let mut ev_option = Some(event);

            while let Some(ev) = ev_option {
                if let Err(e) = self.handle_event(ev) {
                    eprintln!("{}", e);
                }
                ev_option = self.state.connection().poll_for_event()?;
            }
        }
    }

    /// Event handler. Decide what to do with incoming X11 Events.
    fn handle_event(&mut self, event: Event) -> WmResult {
        match event {
            Event::Error(e) => {
                let extension_name = e.extension_name.unwrap_or_else(|| "Unknown".to_string());
                let request_name = e.request_name.unwrap_or("Unknown");
                eprintln!(
                    "[ERR] X11 Error Event Received: error-kind -> {:?},
                          error-code -> {},
                          bad-value -> {},
                          extension-name -> {},
                          request-name -> {}",
                    e.error_kind, e.error_code, e.bad_value, extension_name, request_name
                )
            }
            Event::KeyPress(e) => {
                self.state.handle_key_press(&e)?;
            }

            Event::KeyRelease(e) => self.state.handle_key_release(&e)?,
            Event::MapRequest(e) => {
                self.state.manage_window(e.window)?;
            }
            Event::EnterNotify(e) => {
                #[cfg(debug_assertions)]
                println!("entering window: {}", e.event);
                self.state.handle_enter_event(e.event)?;
            }
            Event::LeaveNotify(_) => {}
            Event::MotionNotify(e) => {
                self.state.handle_motion_notify(&e)?;
            }
            Event::ButtonPress(e) => {
                self.state.handle_button_press(&e)?;
            }
            Event::ButtonRelease(e) => {
                self.state.handle_button_release(&e)?;
            }
            Event::FocusIn(e) => {
                self.state.handle_focus_in(&e)?;
            }
            Event::ClientMessage(_e) => {
                #[cfg(debug_assertions)]
                println!("client message: {}", _e.window);
            }
            Event::Expose(_e) => {
                #[cfg(debug_assertions)]
                println!("expose event on window: {}", _e.window);
            }
            Event::UnmapNotify(_e) => {
                #[cfg(debug_assertions)]
                println!("unmap event: {}", _e.window)
            }
            Event::DestroyNotify(e) => {
                self.state.unmanage_window(e.window)?;
            }
            Event::PropertyNotify(e) => {
                let bar_widnows = self.state.bar_windows();
                if bar_widnows.contains(&e.window) {
                    self.state.update_bars()?;
                } else {
                    #[cfg(debug_assertions)]
                    println!("property notify in window: {} atom: {}", e.window, e.atom);
                }
            }
            _ev => {}
        };

        Ok(())
    }
}
