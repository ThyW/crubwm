use x11rb::{connection::Connection, protocol::Event};

use crate::{
    config::Config,
    errors::WmResult,
    parsers::{Command, CommandType},
    wm::state::State,
};

pub mod actions;
mod container;
mod geometry;
mod layout;
mod state;
mod workspace;

fn print_help_message() {
    println!("crubwm is a tiling X window manager.\n");
    println!("Here is a list of all possible command line options:\n");
    println!("  -h or --help   \t\tPrint this message.");
    println!("  --config [PATH]\t\tUse a different config file.");
}

#[allow(dead_code)]
/// The WM struct, holding all the necessary state and information for and about the opperation of
/// the window manager.
pub struct Wm {
    /// The configuration of the window manager, holding all keybinds, hooks and settings.
    config: crate::config::Config,
    /// Window manager's state. Holds information about X server connection, clients, workspaces
    /// geomoteries, etc...
    state: State,
}

impl Wm {
    pub fn new(config: Config) -> WmResult<Self> {
        let display_name = match config.options.display_name.is_empty() {
            true => Some(config.options.display_name.as_str()),
            false => None,
        };

        let state = State::new(display_name)?;
        Ok(Self { config, state })
    }

    pub fn run(&mut self, commands: Vec<Command>) -> WmResult {
        for command in commands {
            if command.get_type() == &CommandType::Help {
                print_help_message();
                return Ok(());
            }
        }

        let conn = self.state.connection();

        loop {
            conn.flush()?;
            let event = conn.wait_for_event()?;

            let mut event_option = Some(event);
            while let Some(ev) = event_option {
                self.handle_event(ev)?;
                event_option = conn.poll_for_event()?;
            }
        }
    }

    pub fn handle_event(&self, event: Event) -> WmResult {
        println!("event");
        match event {
            Event::Error(e) => {
                println!("X11Error: {:?}", e)
            }
            ev => {
                println!("{:#?}", ev);
            }
        };

        Ok(())
    }
}
