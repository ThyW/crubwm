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
    errm,
    errors::WmResult,
    log::{err, log, LL_NORMAL},
    logm,
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

        let xd = "hello";

        logm!(LL_NORMAL, "Hello world {}", xd);

        logm!(
            LL_NORMAL,
            "Setting up bar update thread. Status bars will automatically be updated every second.",
        );
        let _ = spawn(move || {
            let mut last_time = std::time::Instant::now();
            let mut switch = 0;
            loop {
                if last_time.elapsed().as_secs() >= 1 {
                    last_time = std::time::Instant::now();
                    for win in bar_windows.iter() {
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
        logm!(LL_NORMAL, "Starting the event loop.");
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
                    errm!("{}", e);
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
                errm!(
                    "X11 Error Event Received: error-kind -> {:?},
                          error-code -> {},
                          bad-value -> {},
                          extension-name -> {},
                          request-name -> {}",
                    e.error_kind,
                    e.error_code,
                    e.bad_value,
                    extension_name,
                    request_name
                )
            }
            Event::KeyPress(e) => {
                logm!(LL_NORMAL, "Handling key press event on window {}", e.event);
                self.state.handle_key_press(&e)?;
            }

            Event::KeyRelease(e) => {
                logm!(
                    LL_NORMAL,
                    "Handling key release event on window {}",
                    e.detail,
                );
                self.state.handle_key_release(&e)?
            }
            Event::MapRequest(e) => {
                logm!(LL_NORMAL, "Handling a map request for window {}", e.window,);
                self.state.manage_window(e.window)?;
            }
            Event::EnterNotify(e) => {
                logm!(LL_NORMAL, "Handling enter notify for window {}", e.event,);
                self.state.handle_enter_event(e.event)?;
            }
            Event::LeaveNotify(_) => {}
            Event::MotionNotify(e) => {
                logm!(
                    LL_NORMAL,
                    "Handling a motion notify event in window {}",
                    e.event,
                );

                self.state.handle_motion_notify(&e)?;
            }
            Event::ButtonPress(e) => {
                logm!(
                    LL_NORMAL,
                    "Handling a button press event in window {}",
                    e.event,
                );
                self.state.handle_button_press(&e)?;
            }
            Event::ButtonRelease(e) => {
                logm!(
                    LL_NORMAL,
                    "Handling a button release event in window {}",
                    e.event,
                );
                self.state.handle_button_release(&e)?;
            }
            Event::FocusIn(e) => {
                logm!(LL_NORMAL, "Handling a focus in event in window {}", e.event,);
                self.state.handle_focus_in(&e)?;
            }
            Event::ClientMessage(e) => {
                logm!(
                    LL_NORMAL,
                    "Received a client message from window {}",
                    e.window,
                );
            }
            Event::Expose(e) => {
                logm!(LL_NORMAL, "Exposure event on window {}", e.window,);
            }
            Event::UnmapNotify(_e) => {
                logm!(LL_NORMAL, "Window {} has been unmapped", _e.window,);
            }
            Event::DestroyNotify(e) => {
                logm!( LL_NORMAL, "Window {} has been destroyed, this window will no longer be managed by the window manager.", e.window);
                self.state.unmanage_window(e.window)?;
            }
            Event::PropertyNotify(e) => {
                let bar_widnows = self.state.bar_windows();
                if bar_widnows.contains(&e.window) {
                    self.state.update_bars()?;
                } else {
                    logm!(
                        LL_NORMAL,
                        "property notify in window: {} atom: {}",
                        e.window,
                        e.atom,
                    );
                }
            }
            _ev => {}
        };

        Ok(())
    }
}
