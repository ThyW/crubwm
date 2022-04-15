use x11rb::{
    connection::Connection,
    protocol::Event,
};

use crate::{
    config::Config,
    errors::WmResult,
    parsers::{Command, CommandType},
    wm::state::State,
};

pub mod actions;
mod container;
mod geometry;
mod keyman;
mod layouts;
mod state;
mod workspace;

fn print_help_message() {
    println!("crubwm is a tiling X window manager.\n");
    println!("Here is a list of all possible command line options:\n");
    println!("  -h or --help   \t\tPrint this message.");
    println!("  --config [PATH]\t\tUse a different config file.");
}

#[allow(dead_code)]
/// The WM struct, holding all the necessary state and information for and about the operation of
/// the window manager.
pub struct Wm {
    /// The configuration of the window manager, holding all keybinds, hooks and settings.
    config: crate::config::Config,
    /// Window manager's state. Holds information about X server connection, clients, workspaces
    /// geometries, etc...
    state: State,
}

impl Wm {
    /// Create a new window manager instance.
    pub fn new(config: Config) -> WmResult<Self> {
        let display_name = match config.options.display_name.is_empty() {
            true => Some(config.options.display_name.as_str()),
            false => None,
        };

        // create the state manager here.
        let state = State::new(display_name)?;

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

        // instantiate workspaces
        self.state.init_workspaces();
        // check for all open windows and manage them
        self.state.become_wm()?;
        // notify the window manager of the keybinds
        self.state.init_keyman(self.config.keybinds.clone())?;

        loop {
            self.state.connection().flush()?;
            let event = self.state.connection().wait_for_event()?;

            let mut event_option = Some(event);
            while let Some(ev) = event_option {
                if let Err(e) = self.handle_event(ev) {
                    println!("{}", e)
                }
                event_option = self.state.connection().poll_for_event()?;
            }
        }
    }

    /// Event handler. Decide what to do with incoming X11 Events.
    fn handle_event(&mut self, event: Event) -> WmResult {
        match event {
            Event::Error(e) => {
                println!("X11Error: {:?}", e)
            }
            Event::KeyPress(e) => self.state.handle_key_press(&e)?,

            Event::KeyRelease(e) => self.state.handle_key_release(&e)?,
            Event::CreateNotify(e) => {
                println!("root window geometry: {}", self.state.root_geometry()?);
                self.state.manage_window(e.window)?;
            }
            Event::EnterNotify(e) => {
                self.state.handle_enter_event(e.event)?;
            }
            Event::Expose(e) => {
                println!("expose event on window: {}", e.window);
            }
            Event::UnmapNotify(e) => {
                println!("unmap event: {:#?}", e)
            }
            Event::DestroyNotify(e) => {
                self.state.unmanage_window(e.window)?;
            }
            _ev => {}
        };

        Ok(())
    }
}
