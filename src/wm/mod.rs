use std::rc::Rc;

use x11rb::{connection::Connection, protocol::Event};

use crate::{
    config::Config,
    errors::WmResult,
    parsers::{Command, CommandType},
    wm::state::State,
};

pub mod actions;
pub mod atoms;
pub mod container;
pub mod focus_stack;
pub mod geometry;
pub mod keyman;
pub mod layouts;
pub mod state;
pub mod workspace;

fn print_help_message() {
    println!("crubwm is a tiling X window manager.\n");
    println!("Here is a list of all command line options:\n");
    println!("  -h or --help   \t\tPrint this message.");
    println!("  --config [PATH]\t\tUse a different config file.");
}

#[allow(dead_code)]
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
        let display_name = match c.options.display_name.is_empty() {
            false => Some(c.options.display_name.as_str()),
            true => None,
        };

        println!("{:#?}", config.workspace_settings.get()[0]);
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
        // check for all open windows and manage them
        self.state.become_wm()?;
        // notify the window manager of the keybinds
        self.state.init_keyman(self.config.keybinds.clone())?;

        // run the event loop, don't stop on errors, just report them and keep going.
        loop {
            self.state.connection().flush()?;
            let event = self.state.connection().wait_for_event()?;

            let mut event_option = Some(event);
            while let Some(ev) = event_option {
                if let Err(e) = self.handle_event(ev) {
                    eprintln!("{e}")
                }
                event_option = self.state.connection().poll_for_event()?;
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
                println!("map request");
                self.state.manage_window(e.window)?;
            }
            Event::EnterNotify(e) => {
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
            Event::PropertyNotify(_e) => {
                #[cfg(debug_assertions)]
                println!("property notify in window: {} atom: {}", _e.window, _e.atom);
            }
            _ev => {}
        };

        Ok(())
    }
}
